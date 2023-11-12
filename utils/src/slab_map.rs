use std::borrow::Borrow;
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use bimap::BiHashMap;
use nohash_hasher::NoHashHasher;
use serde::Deserializer;
use slab::Slab;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SlabMapId<V>(usize, PhantomData<V>);

impl<V> SlabMapId<V> {
    fn new(id: usize) -> Self {
        Self(id, Default::default())
    }

    pub fn raw(&self) -> usize {
        self.0
    }

    pub fn as_untyped(&self) -> SlabMapUntypedId {
        SlabMapUntypedId::new(self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SlabMapUntypedId(usize);

impl SlabMapUntypedId {
    fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> usize {
        self.0
    }

    /// Performs unchecked conversion into a typed slab map ID
    ///
    /// Indexing directly with a resulting ID might lead to panics if the
    /// original ID did not belong to the indexed SlabMap
    pub fn as_typed_unchecked<T>(&self) -> SlabMapId<T> {
        SlabMapId::new(self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SlabMapKeyOrId<K, V> {
    Id(SlabMapId<V>),
    Key(K),
}

impl<K, V> From<SlabMapId<V>> for SlabMapKeyOrId<K, V> {
    fn from(value: SlabMapId<V>) -> Self {
        SlabMapKeyOrId::Id(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SlabMapKeyOrUntypedId<K> {
    Id(SlabMapUntypedId),
    Key(K),
}

impl<K> From<SlabMapUntypedId> for SlabMapKeyOrUntypedId<K> {
    fn from(value: SlabMapUntypedId) -> Self {
        SlabMapKeyOrUntypedId::Id(value)
    }
}

#[derive(Debug, Clone)]
pub struct SlabMap<K: Eq + Hash, V, Hasher: BuildHasher = BuildHasherDefault<rustc_hash::FxHasher>>
{
    items: Slab<V>,
    keys: BiHashMap<K, usize, Hasher, BuildHasherDefault<NoHashHasher<usize>>>,
}

impl<K: Eq + Hash, V, Hasher: BuildHasher> SlabMap<K, V, Hasher> {
    pub fn insert(&mut self, key: K, value: V) -> (SlabMapId<V>, Option<V>) {
        match self.keys.get_by_left(&key) {
            None => {
                let id = self.items.insert(value);
                self.keys.insert(key, id);

                (SlabMapId::new(id), None)
            }
            Some(id) => {
                let mut old = value;
                std::mem::swap(&mut self.items[*id], &mut old);
                (SlabMapId::new(*id), Some(old))
            }
        }
    }

    pub fn get_by_id(&self, id: SlabMapId<V>) -> Option<&V> {
        self.items.get(id.0)
    }

    pub fn get_by_id_mut(&mut self, id: SlabMapId<V>) -> Option<&mut V> {
        self.items.get_mut(id.0)
    }

    pub fn get_by_untyped_id(&self, id: SlabMapUntypedId) -> Option<&V> {
        self.items.get(id.0)
    }

    pub fn get_by_untyped_id_mut(&mut self, id: SlabMapUntypedId) -> Option<&mut V> {
        self.items.get_mut(id.0)
    }

    pub fn get_by_raw(&self, id: usize) -> Option<&V> {
        self.items.get(id)
    }

    pub fn get_by_raw_mut(&mut self, id: usize) -> Option<&mut V> {
        self.items.get_mut(id)
    }

    pub fn get_by_key(&self, key: &K) -> Option<&V> {
        self.keys.get_by_left(key).and_then(|e| self.get_by_raw(*e))
    }

    pub fn get_by_key_mut(&mut self, key: &K) -> Option<&mut V> {
        self.keys
            .get_by_left(key)
            .copied()
            .and_then(|e| self.get_by_raw_mut(e))
    }

    pub fn get<Q: Borrow<K>>(&self, k: SlabMapKeyOrId<Q, V>) -> Option<&V> {
        match k {
            SlabMapKeyOrId::Id(id) => self.get_by_id(id),
            SlabMapKeyOrId::Key(key) => self.get_by_key(key.borrow()),
        }
    }

    pub fn get_mut<Q: Borrow<K>>(&mut self, k: SlabMapKeyOrId<Q, V>) -> Option<&mut V> {
        match k {
            SlabMapKeyOrId::Id(id) => self.get_by_id_mut(id),
            SlabMapKeyOrId::Key(key) => self.get_by_key_mut(key.borrow()),
        }
    }

    pub fn get_by_untyped<Q: Borrow<K>>(&self, k: SlabMapKeyOrUntypedId<Q>) -> Option<&V> {
        match k {
            SlabMapKeyOrUntypedId::Id(id) => self.get_by_raw(id.0),
            SlabMapKeyOrUntypedId::Key(key) => self.get_by_key(key.borrow()),
        }
    }

    pub fn get_by_untyped_mut<Q: Borrow<K>>(
        &mut self,
        k: SlabMapKeyOrUntypedId<Q>,
    ) -> Option<&mut V> {
        match k {
            SlabMapKeyOrUntypedId::Id(id) => self.get_by_raw_mut(id.0),
            SlabMapKeyOrUntypedId::Key(key) => self.get_by_key_mut(key.borrow()),
        }
    }

    pub fn key_to_id<Q: Borrow<K>>(&self, key: Q) -> Option<SlabMapId<V>> {
        self.keys
            .get_by_left(key.borrow())
            .map(|e| SlabMapId::new(*e))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items.iter().map(|e| e.1)
    }
}

impl<K: Eq + Hash, V, Hasher: BuildHasher> Index<SlabMapId<V>> for SlabMap<K, V, Hasher> {
    type Output = V;

    fn index(&self, index: SlabMapId<V>) -> &Self::Output {
        &self.items[index.0]
    }
}

impl<K: Eq + Hash, V, Hasher: BuildHasher> IndexMut<SlabMapId<V>> for SlabMap<K, V, Hasher> {
    fn index_mut(&mut self, index: SlabMapId<V>) -> &mut Self::Output {
        &mut self.items[index.0]
    }
}

impl<K: Eq + Hash, V, Hasher: BuildHasher + Default> Default for SlabMap<K, V, Hasher> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            keys: Default::default(),
        }
    }
}

impl<'de, K: serde::Deserialize<'de>, V> serde::Deserialize<'de> for SlabMapKeyOrId<K, V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        K::deserialize(deserializer).map(|k| Self::Key(k))
    }
}
