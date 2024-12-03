use std::{
    any::Any,
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::property::{
    backend::{Operation, PropertyBackend},
    PropertyName,
};

#[derive(Debug)]
pub struct PNBackend {
    values: Arc<RwLock<BTreeMap<PropertyName, PNValue>>>,
}

#[derive(Debug)]
pub struct PNValue {
    // TODO: Maybe use something aside from i64 for the base?
    pub value: i64,
    // TODO: replace with precursor record?
    pub previous_value: i64,
}

impl PNValue {
    pub fn snapshot(&self) -> Self {
        Self {
            value: self.value,
            previous_value: self.value,
        }
    }

    pub fn diff(&self) -> i64 {
        self.value - self.previous_value
    }
}

impl Default for PNValue {
    fn default() -> Self {
        Self {
            value: 0,
            previous_value: 0,
        }
    }
}

impl PNBackend {
    pub fn new() -> PNBackend {
        Self {
            values: Arc::new(RwLock::new(BTreeMap::default())),
        }
    }

    pub fn add(&self, property_name: PropertyName, amount: i64) {
        let values = self.values.write().unwrap();
        Self::add_raw(values, property_name, amount);
    }

    pub fn add_raw(
        mut values: impl DerefMut<Target = BTreeMap<PropertyName, PNValue>>,
        property_name: PropertyName,
        amount: i64,
    ) {
        let value = values
            .deref_mut()
            .entry(property_name)
            .or_insert(PNValue::default());
        value.value += amount;
    }
}

impl PropertyBackend for PNBackend {
    fn as_arc_dyn_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> {
        self as Arc<dyn Any + Send + Sync + 'static>
    }

    fn as_debug(&self) -> &dyn Debug {
        self as &dyn Debug
    }

    fn duplicate(&self) -> Box<dyn PropertyBackend> {
        let values = self.values.read().unwrap();
        let snapshotted = values
            .iter()
            .map(|(key, value)| (key.to_owned(), value.snapshot()))
            .collect::<BTreeMap<_, _>>();
        Box::new(Self {
            values: Arc::new(RwLock::new(snapshotted)),
        })
    }

    fn property_backend_name() -> String {
        "pn".to_owned()
    }

    fn to_state_buffer(&self) -> anyhow::Result<Vec<u8>> {
        let values = self.values.read().unwrap();
        let serializable = values
            .iter()
            .map(|(key, value)| (key, value.value))
            .collect::<BTreeMap<_, _>>();
        let serialized = bincode::serialize(&serializable)?;
        Ok(serialized)
    }

    fn from_state_buffer(
        state_buffer: &Vec<u8>,
    ) -> std::result::Result<Self, crate::error::RetrievalError> {
        let values = bincode::deserialize::<BTreeMap<PropertyName, i64>>(state_buffer)?;
        let values = values
            .into_iter()
            .map(|(key, value)| {
                (
                    key,
                    PNValue {
                        value: value,
                        previous_value: value,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        Ok(Self {
            values: Arc::new(RwLock::new(values)),
        })
    }

    fn to_operations(&self /*precursor: ULID*/) -> anyhow::Result<Vec<Operation>> {
        let values = self.values.read().unwrap();
        let diffs = values
            .iter()
            .map(|(key, value)| (key, value.diff()))
            .collect::<BTreeMap<_, _>>();

        let serialized_diffs = bincode::serialize(&diffs)?;
        Ok(vec![Operation {
            diff: serialized_diffs,
        }])
    }

    fn apply_operations(&self, operations: &Vec<Operation>) -> anyhow::Result<()> {
        for operation in operations {
            let diffs = bincode::deserialize::<BTreeMap<PropertyName, i64>>(&operation.diff)?;

            let mut values = self.values.write().unwrap();
            for (property, diff) in diffs {
                Self::add_raw(&mut *values, property, diff);
            }
        }

        Ok(())
    }
}
