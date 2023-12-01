use bevy::prelude::Component;
use bevy::utils::thiserror::Error;
use ehce_core::database::model::formula::Formula;
use ehce_core::database::model::ItemId;
use itertools::Itertools;
use miette::Diagnostic;
use nohash_hasher::IntMap;
use soa_derive::StructOfArray;

use std::sync::Arc;

use ehce_core::database::model::resource::ResourceId;
use ehce_core::mods::ModData;

/// Component to track entity resources
///
/// Computed resource dependencies must form an
/// [acyclic graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph),
/// otherwise an error will be returned
#[derive(Debug, Clone, Default, Component)]
pub struct Resources {
    /// Mapping of resource ID to internal ID
    ids: IntMap<ResourceId, usize>,
    data: ResourceGraphVec,
    in_progress: Vec<ResourceId>,
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
            let id = resources.get_id_or_init(res, db)?;
            resources.data.value[id] += amount;
        }

        Ok(resources)
    }

    /// Calculates value of the resource, inserting it if not present
    pub fn calculate(
        &mut self,
        db: &ModData,
        res_id: ResourceId,
    ) -> Result<f64, ResourceEvaluationError> {
        let id = self.get_id_or_init(res_id, db)?;
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
        let id = self.get_id_or_init(res_id, db)?;
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
        let id = self.get_id_or_init(res_id, db)?;
        Self::invalidate_cache(&mut self.data.cache, &self.data.rdeps, id);
        self.data.value[id] += value;
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
                Err(err) => {
                    return Err(ResourceEvaluationError::EvaluationError(
                        err,
                        debug_key(db, res_id),
                    ))
                }
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
        resource_id: ResourceId,
        db: &ModData,
    ) -> Result<usize, ResourceEvaluationError> {
        if let Some(id) = self.ids.get(&resource_id) {
            return Ok(*id);
        }

        let res = &db.registry.resource[resource_id];
        let id = self.data.len();
        self.data.push(ResourceGraph {
            resource_id,
            cache: None,
            value: 0.0,
            formula: res.computed.clone(),
            deps: vec![],
            rdeps: vec![],
        });

        let other = self.ids.insert(resource_id, id);
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
                return Err(ResourceEvaluationError::CircularDependencyError(slice));
            }
            Ok(())
        }

        if res.computed.is_some() || res.default.is_some() {
            self.in_progress.push(resource_id);

            if let Some(computed) = &res.computed {
                for arg in &computed.args {
                    check_deps(&mut self.in_progress, db, arg)?;

                    let dep_id = self.get_id_or_init(*arg, db)?;
                    self.add_dep(id, dep_id);
                }
            }

            if let Some(default) = &res.default {
                let mut args = Vec::with_capacity(default.args.len());
                for arg in &default.args {
                    check_deps(&mut self.in_progress, db, arg)?;

                    let value = self.calculate(db, *arg)?;

                    args.push(value)
                }

                let default = default.expr.eval_vec(args).map_err(|e| {
                    ResourceEvaluationError::DefaultEvaluationError(e, debug_key(db, resource_id))
                })?;
                self.data.value[id] = default;
            }

            self.in_progress.pop();
        }

        Ok(id)
    }

    fn add_dep(&mut self, origin: usize, dependency: usize) {
        self.data.deps[origin].push(dependency);
        self.data.rdeps[dependency].push(origin);
    }
}

fn debug_key(db: &ModData, id: ResourceId) -> ItemId {
    db.registry
        .resource
        .id_to_key(id)
        .cloned()
        .unwrap_or_else(|| format!("{:?}", id))
}

#[derive(Debug, Clone, Error, Diagnostic)]
pub enum ResourceEvaluationError {
    #[error("Failed to evaluate Resource({}): {}", .1, .0)]
    EvaluationError(exmex::ExError, ItemId),
    #[error("Failed to evaluate default value for Resource({}): {}", .1, .0)]
    DefaultEvaluationError(exmex::ExError, ItemId),
    #[error("Circular dependency while evaluating the resource. Stack: [{}]", .0.join(", "))]
    CircularDependencyError(Vec<ItemId>),
}
