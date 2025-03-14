use std::{
    any::Any,
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    property::{backend::PropertyBackend, PropertyName},
    storage::Materialized,
};

#[derive(Clone, Debug)]
pub struct LWWBackend {
    // TODO: store a timestamp/precursor id at time of setting value (or maybe when we commit?).
    values: Arc<RwLock<BTreeMap<PropertyName, Vec<u8>>>>,
}

impl Default for LWWBackend {
    fn default() -> Self { Self::new() }
}

impl LWWBackend {
    pub fn new() -> LWWBackend { Self { values: Arc::new(RwLock::new(BTreeMap::default())) } }

    pub fn set(&mut self, property_name: PropertyName, value: Vec<u8>) {
        let mut values = self.values.write().unwrap();
        match values.entry(property_name) {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }

    pub fn get(&self, property_name: PropertyName) -> Option<Vec<u8>> {
        let values = self.values.read().unwrap();
        values.get(&property_name).cloned()
    }
}

impl PropertyBackend for LWWBackend {
    fn as_arc_dyn_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> { self as Arc<dyn Any + Send + Sync + 'static> }

    fn as_debug(&self) -> &dyn Debug { self as &dyn Debug }

    fn fork(&self) -> Box<dyn PropertyBackend> {
        let values = self.values.read().unwrap();
        let cloned = (*values).clone();
        drop(values);

        Box::new(Self { values: Arc::new(RwLock::new(cloned)) })
    }

    fn properties(&self) -> Vec<String> {
        let values = self.values.read().unwrap();
        values.keys().cloned().collect::<Vec<String>>()
    }

    fn materialized(&self) -> BTreeMap<PropertyName, Materialized> { unimplemented!() }

    fn property_backend_name() -> String { "lww".to_owned() }

    fn to_state_buffer(&self) -> anyhow::Result<Vec<u8>> {
        let values = self.values.read().unwrap();
        let state_buffer = bincode::serialize(&*values)?;
        Ok(state_buffer)
    }

    fn from_state_buffer(state_buffer: &Vec<u8>) -> std::result::Result<Self, crate::error::RetrievalError>
    where Self: Sized {
        let map = bincode::deserialize::<BTreeMap<PropertyName, Vec<u8>>>(state_buffer)?;
        Ok(Self { values: Arc::new(RwLock::new(map)) })
    }

    fn to_operations(&self /*precursor: ULID*/) -> anyhow::Result<Vec<super::Operation>> { todo!() }

    fn apply_operations(&self, _operations: &Vec<super::Operation>) -> anyhow::Result<()> { todo!() }

    fn get_property_value_string(&self, property_name: &str) -> Option<String> {
        self.values.read().unwrap().get(property_name).map(|v| String::from_utf8_lossy(v).to_string())
    }
}

// Need ID based happens-before determination to resolve conflicts
