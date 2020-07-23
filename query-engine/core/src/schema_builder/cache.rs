//! Every builder is required to cache input and output type refs that are created inside
//! of them, e.g. as soon as the builder produces a ref, it must be retrievable later,
//! without leaking memory due to Arcs pointing to each other.
//!
//! The cache has two purposes:
//! - First, break circular dependencies, as they can happen in recursive input / output types.
//! - Second, it serves as a central list of build types of that builder, which are used later to
//!   collect all types of the query schema.
//!
//! The cached types are stored as Arcs, and the cache owns these (strong) Arcs,
//! while the cache will only hand out weak arcs. Not only does this simplify the builder architecture,
//! but also prevents issues with memory leaks in the schema, as well as issues that when all strong
//! arcs are dropped due to visitor operations, the schema can't be traversed anymore due to invalid references.
use super::*;
use std::{collections::HashMap, fmt::Debug, sync::Weak};

/// Cache wrapper over Arc<T>.
/// Caches keys at most once, and errors on repeated insertion of the same key
/// to uphold schema building consistency guarantees.
#[derive(Debug, Default)]
pub struct TypeRefCache<T> {
    cache: HashMap<String, Arc<T>>,
}

impl<T: Debug> TypeRefCache<T> {
    pub fn new() -> Self {
        TypeRefCache { cache: HashMap::new() }
    }

    // Retrieves a cached Arc if present, and hands out a weak reference to the contents.
    pub fn get(&self, key: &str) -> Option<Weak<T>> {
        self.cache.get(key).map(|v| Arc::downgrade(v))
    }

    /// Caches given value with given key. Panics if the cache key already exists.
    /// The reason is that for the query schema to work, we need weak references to be valid,
    /// which might be violated if we insert a new arc into the cache that replaces the old one,
    /// as it invalidates all weak refs pointing to the replaced arc, assuming that the contents
    /// changed as well. While this restriction could be lifted by comparing the contents, it is
    /// not required in the context of the schema builders.
    pub fn insert(&mut self, key: String, value: Arc<T>) {
        if let Some(old) = self.cache.insert(key.clone(), value) {
            panic!(format!(
                "Invariant violation: Inserted key {} twice, this is a bug and invalidates weak arc references. {:?}",
                key, old
            ))
        }
    }
}

/// Consumes the cache and returns all contents as vector of the cached values.
impl<T> Into<Vec<Arc<T>>> for TypeRefCache<T> {
    fn into(self) -> Vec<Arc<T>> {
        let mut vec: Vec<Arc<T>> = vec![];

        vec.extend(self.cache.into_iter().map(|(_, t)| t));
        vec
    }
}

/// Builds a cache over T from a vector of tuples of shape (String, Arc<T>).
impl<T> From<Vec<(String, Arc<T>)>> for TypeRefCache<T> {
    fn from(tuples: Vec<(String, Arc<T>)>) -> TypeRefCache<T> {
        TypeRefCache {
            cache: tuples.into_iter().collect(),
        }
    }
}

/// Convenience cache utility to load and return immediately if a type is already cached.
macro_rules! return_cached {
    ($cache:expr, $name:expr) => {
        let existing_type = $cache.get($name);
        if existing_type.is_some() {
            return existing_type.unwrap();
        }
    };
}
