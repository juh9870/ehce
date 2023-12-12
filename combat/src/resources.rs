use bevy::prelude::Component;
use bevy::utils::thiserror::Error;
use ehce_core::database::model::formula::Formula;
use ehce_core::database::model::{ItemId, ResourceId};
use itertools::Itertools;
use miette::Diagnostic;
use nohash_hasher::IntMap;
use soa_derive::StructOfArray;

use std::sync::{Arc, Mutex};

use ehce_core::mods::ModData;

/// Component to track entity resources
///
/// Computed resource dependencies must form an
/// [acyclic graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph),
/// otherwise an error will be returned
#[derive(Debug, Default, Component)]
pub struct Resources {
    /// Mapping of resource ID to internal ID
    ids: IntMap<ResourceId, usize>,
    /// Cache of default computed values
    wanted_cache: Mutex<IntMap<ResourceId, f64>>,
    data: ResourceGraphVec,
    in_progress: Vec<ResourceId>,
}

impl Clone for Resources {
    fn clone(&self) -> Self {
        Self {
            ids: self.ids.clone(),
            wanted_cache: Mutex::new(self.wanted_cache.lock().unwrap().clone()),
            data: self.data.clone(),
            in_progress: self.in_progress.clone(),
        }
    }
}

#[derive(Debug, Clone, StructOfArray)]
#[soa_derive(Debug)]
#[soa_attr(Vec, derive(Default, Clone))]
struct ResourceGraph {
    /// DB id of the resource
    resource_id: ResourceId,
    /// Cache of computes values, invalidated on change
    cache: Option<f64>,
    /// "raw" value. Returned directly for non-computed resources, and added
    /// to the computed result5 for computed resources
    value: f64,
    /// reference to the formula used to compute the value
    formula: Option<Arc<Formula>>,
    /// Dependencies of the computed resource
    deps: Vec<usize>,
    /// Resources that depend on this resource, used for invalidating cache
    rdeps: Vec<usize>,
}

impl Resources {
    pub fn from_stats(
        db: &ModData,
        stats: impl IntoIterator<Item = (ResourceId, f64)>,
    ) -> Result<Self, ResourceEvaluationError> {
        let mut resources = Self::default();

        for (res, amount) in stats {
            let id = resources.get_id_or_init(db, res)?;
            resources.data.value[id] += amount;
        }

        Ok(resources)
    }

    /// Calculates value of the resource. Missing resources will get cached,
    /// but won't be fully inserted
    /// TODO: add cyclical dependencies handling
    pub fn calculate(
        &self,
        db: &ModData,
        res_id: ResourceId,
    ) -> Result<f64, ResourceEvaluationError> {
        let (formula, value) = if let Some(id) = self.ids.get(&res_id) {
            if let Some(cached) = self.data.cache[*id] {
                return Ok(cached);
            }

            (&self.data.formula[*id], self.data.value[*id])
        } else {
            if let Some(cached) = self.wanted_cache.lock().unwrap().get(&res_id) {
                return Ok(*cached);
            }
            let res = &db.registry[res_id];

            let default = if let Some(default) = &res.data.default {
                let args = default
                    .args
                    .iter()
                    .map(|e| self.calculate(db, *e))
                    .try_collect()?;
                match default.expr.eval_vec(args) {
                    Ok(data) => data,
                    Err(err) => return Err(EvaluationError(err, debug_key(db, res_id)).into()),
                }
            } else {
                0.0
            };

            (&res.data.computed, default)
        };

        if let Some(formula) = formula {
            let args = formula
                .args
                .iter()
                .map(|e| self.calculate(db, *e))
                .try_collect()?;
            match formula.expr.eval_vec(args) {
                Ok(value) => Ok(value),
                Err(err) => Err(EvaluationError(err, debug_key(db, res_id)).into()),
            }
        } else {
            Ok(value)
        }
    }

    /// Calculates value of the resource, inserting it if not present, or
    /// updating it if not cached
    pub fn calculate_mut(
        &mut self,
        db: &ModData,
        res_id: ResourceId,
    ) -> Result<f64, ResourceEvaluationError> {
        let id = self.get_id_or_init(db, res_id)?;
        Self::calculate_inner(
            db,
            &self.data.resource_id,
            &self.data.value,
            &mut self.data.cache,
            &self.data.deps,
            &self.data.formula,
            id,
            res_id,
        )
    }

    /// Sets raw value of the specified resource, inserting it if not present
    pub fn set(
        &mut self,
        db: &ModData,
        res_id: ResourceId,
        value: f64,
    ) -> Result<(), ResourceEvaluationError> {
        let id = self.get_id_or_init(db, res_id)?;
        Self::invalidate_cache(&mut self.data.cache, &self.data.rdeps, id);
        self.data.value[id] = value;
        Ok(())
    }

    /// Increases raw value of the specified resource by a given amount
    pub fn add(
        &mut self,
        db: &ModData,
        res_id: ResourceId,
        value: f64,
    ) -> Result<(), ResourceEvaluationError> {
        let id = self.get_id_or_init(db, res_id)?;
        Self::invalidate_cache(&mut self.data.cache, &self.data.rdeps, id);
        self.data.value[id] += value;
        Ok(())
    }

    /// Calculates cache for all "dirty" resources, as well as flushes
    /// [calculate] cache
    pub fn recalculate_dirty(&mut self, db: &ModData) -> Result<(), ResourceEvaluationError> {
        self.process_calculation_cache(db)?;

        let mut i = 0;
        while i < self.data.len() {
            if self.data.cache[i].is_none() {
                self.calculate_mut(db, self.data.resource_id[i])?;
            }
            i += 1;
        }

        Ok(())
    }

    /// Clears [calculate] cache and initializes all accessed resources
    pub fn process_calculation_cache(
        &mut self,
        db: &ModData,
    ) -> Result<(), ResourceEvaluationError> {
        let mut cache = self.wanted_cache.lock().unwrap();
        for id in cache.keys() {
            Self::get_id_or_init_raw(
                db,
                &mut self.ids,
                &mut self.data,
                &mut self.in_progress,
                *id,
            )?;
        }
        cache.clear();

        Ok(())
    }

    /// Clears all resources stored in a map
    pub fn clear(&mut self) {
        self.ids.clear();
        self.data.clear();
    }

    fn calculate_inner(
        db: &ModData,
        rids: &[ResourceId],
        values: &[f64],
        cache: &mut [Option<f64>],
        deps: &[Vec<usize>],
        formulas: &[Option<Arc<Formula>>],
        id: usize,
        res_id: ResourceId,
    ) -> Result<f64, ResourceEvaluationError> {
        if let Some(cached) = &cache[id] {
            return Ok(*cached);
        }

        let raw_value = values[id];
        let value = if let Some(formula) = &formulas[id] {
            let arguments: Vec<f64> = deps[id]
                .iter()
                .map(|dep_id| {
                    Self::calculate_inner(
                        db, rids, values, cache, deps, formulas, *dep_id, rids[id],
                    )
                })
                .try_collect()?;
            match formula.expr.eval_vec(arguments) {
                Ok(data) => data + raw_value,
                Err(err) => return Err(EvaluationError(err, debug_key(db, res_id)).into()),
            }
        } else {
            raw_value
        };

        cache[id] = Some(value);

        Ok(value)
    }

    fn invalidate_cache(cache: &mut [Option<f64>], rdeps: &[Vec<usize>], id: usize) {
        cache[id] = None;
        for id in &rdeps[id] {
            Self::invalidate_cache(cache, rdeps, *id)
        }
    }

    fn get_id_or_init(
        &mut self,
        db: &ModData,
        resource_id: ResourceId,
    ) -> Result<usize, ResourceEvaluationError> {
        Self::get_id_or_init_raw(
            db,
            &mut self.ids,
            &mut self.data,
            &mut self.in_progress,
            resource_id,
        )
    }

    fn get_id_or_init_raw(
        db: &ModData,
        ids: &mut IntMap<ResourceId, usize>,
        data: &mut ResourceGraphVec,
        in_progress: &mut Vec<ResourceId>,
        resource_id: ResourceId,
    ) -> Result<usize, ResourceEvaluationError> {
        if let Some(id) = ids.get(&resource_id) {
            return Ok(*id);
        }

        let res = &db.registry.resource[resource_id];
        let id = data.len();
        data.push(ResourceGraph {
            resource_id,
            cache: None,
            value: 0.0,
            formula: res.data.computed.clone(),
            deps: vec![],
            rdeps: vec![],
        });

        let other = ids.insert(resource_id, id);
        debug_assert!(other.is_none(), "Id should be new. id={:?}", resource_id);

        #[inline(always)]
        fn check_deps(
            in_progress: &mut Vec<ResourceId>,
            db: &ModData,
            res: &ResourceId,
        ) -> Result<(), ResourceEvaluationError> {
            if let Some(idx) =
                in_progress.iter().enumerate().find_map(
                    |(id, e)| {
                        if e == res {
                            Some(id)
                        } else {
                            None
                        }
                    },
                )
            {
                in_progress.push(*res);
                let slice = in_progress[idx..]
                    .iter()
                    .map(|e| debug_key(db, *e))
                    .collect_vec();
                return Err(CircularDependencyError(slice).into());
            }
            Ok(())
        }

        if res.data.computed.is_some() || res.data.default.is_some() {
            in_progress.push(resource_id);

            if let Some(computed) = &res.data.computed {
                for arg in &computed.args {
                    check_deps(in_progress, db, arg)?;

                    let dep_id = Self::get_id_or_init_raw(db, ids, data, in_progress, *arg)?;
                    Self::add_dep(
                        data.deps.as_mut_slice(),
                        data.rdeps.as_mut_slice(),
                        id,
                        dep_id,
                    );
                }
            }

            if let Some(default) = &res.data.default {
                let mut args = Vec::with_capacity(default.args.len());
                for arg in &default.args {
                    check_deps(in_progress, db, arg)?;

                    let arg_id = Self::get_id_or_init_raw(db, ids, data, in_progress, *arg)?;
                    let value = Self::calculate_inner(
                        db,
                        &data.resource_id,
                        &data.value,
                        &mut data.cache,
                        &data.deps,
                        &data.formula,
                        arg_id,
                        *arg,
                    )?;

                    args.push(value)
                }

                let default = default
                    .expr
                    .eval_vec(args)
                    .map_err(|e| DefaultEvaluationError(e, debug_key(db, resource_id)))?;
                data.value[id] = default;
            }

            in_progress.pop();
        }

        Ok(id)
    }

    fn add_dep(
        deps: &mut [Vec<usize>],
        rdeps: &mut [Vec<usize>],
        origin: usize,
        dependency: usize,
    ) {
        deps[origin].push(dependency);
        rdeps[dependency].push(origin);
    }
}

fn debug_key(db: &ModData, id: ResourceId) -> ItemId {
    db.registry
        .resource
        .id_to_key(id)
        .cloned()
        .unwrap_or_else(|| format!("{:?}", id))
}

utils::bubbled!(ResourceEvaluationError {
    EvaluationError,
    DefaultEvaluationError,
    CircularDependencyError,
});

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Resource {} is dirty", .0)]
pub struct ResourceDirtyError(ItemId);

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Failed to evaluate Resource({}): {}", .1, .0)]
pub struct EvaluationError(exmex::ExError, ItemId);

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Failed to evaluate default value for Resource({}): {}", .1, .0)]
pub struct DefaultEvaluationError(exmex::ExError, ItemId);

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Circular dependency while evaluating the resource. Stack: [{}]", .0.join(", "))]
pub struct CircularDependencyError(Vec<ItemId>);
