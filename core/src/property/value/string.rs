use std::sync::{Arc, Mutex};

use anyhow::Result;

use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    ReadTxn, StateVector, Text, Transact, Update,
};

use crate::{
    model::RecordInner, property::{backend::Yrs, traits::{InitializeWith, StateSync}}, storage::FieldValue
};

#[derive(Debug)]
pub struct StringValue {
    // ideally we'd store the yrs::TransactionMut in the Transaction as an ExtendableOp or something like that
    // and call encode_update_v2 on it when we're ready to commit
    // but its got a lifetime of 'doc and that requires some refactoring
    pub record_inner: Arc<RecordInner>,
    pub property_name: &'static str,
    previous_state: Arc<Mutex<StateVector>>,
    //backend: Arc<crate::property::backend::Yrs>,
}

// Starting with basic string type operations
impl StringValue {
    pub fn new(property_name: &'static str, record_inner: Arc<RecordInner>) -> Self {
        let starting_state = record_inner.yrs.doc.transact().state_vector();
        Self {
            property_name,
            record_inner,
            previous_state: Arc::new(Mutex::new(starting_state)),
        }
    }
    pub fn backend(&self) -> &Yrs {
        &self.record_inner.yrs
    }
    pub fn doc(&self) -> &yrs::Doc {
        &self.backend().doc
    }
    pub fn value(&self) -> String {
        self.backend().get_string(self.property_name)
    }
    pub fn insert(&self, index: u32, value: &str) {
        self.backend().insert(self.property_name, index, value);
    }
    pub fn delete(&self, index: u32, length: u32) {
        self.backend().delete(self.property_name, index, length);
    }
}

impl InitializeWith<String> for StringValue {
    fn initialize_with(
        inner: Arc<RecordInner>,
        property_name: &'static str,
        value: String,
    ) -> Self {
        let new_string = Self::new(property_name, inner);
        new_string.insert(0, &value);
        new_string
    }
}

impl StateSync for StringValue {
    fn field_value(&self) -> FieldValue {
        FieldValue::StringValue
    }
    /// Apply an update to the field from an event/operation
    fn apply_update(&self, update: &[u8]) -> Result<()> {
        let mut txn = self.backend().doc.transact_mut();
        let update = Update::decode_v2(update)?;
        txn.apply_update(update)?;
        Ok(())
    }
    /// Retrieve the current state of the field, suitable for storing in the materialized record
    fn state(&self) -> Vec<u8> {
        let txn = self.backend().doc.transact();
        txn.state_vector().encode_v2()
    }
    /// Retrieve the pending update for this field since the last call to this method
    /// ideally this would be centralized in the TypeModule, rather than having to poll each field
    fn get_pending_update(&self) -> Option<Vec<u8>> {
        // hack until we figure out how to get the transaction mut to live across individual field updates
        // diff the previous state with the current state
        let mut previous_state = self.previous_state.lock().unwrap();

        let txn = self.backend().doc.transact_mut();
        let diff = txn.encode_diff_v2(&previous_state);
        *previous_state = txn.state_vector();

        if diff.is_empty() {
            None
        } else {
            Some(diff)
        }
    }
}
