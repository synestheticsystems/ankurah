use std::sync::{Arc, Mutex};

use anyhow::Result;

#[derive(Debug)]
pub struct StringValue {
    // TODO consolidate
    // engine: crate::types::engine::yrs::Yrs,
    doc: yrs::Doc,
    // ideally we'd store the yrs::TransactionMut in the Transaction as an ExtendableOp or something like that
    // and call encode_update_v2 on it when we're ready to commit
    // but its got a lifetime of 'doc and that requires some refactoring
    pub previous_state: Arc<Mutex<StateVector>>,
    pub record_inner: Arc<RecordInner>,
    pub property_name: &'static str,
}

use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    GetString, ReadTxn, StateVector, Text, Transact, Update,
};

use crate::{
    model::RecordInner,
    types::traits::{InitializeWith, StateSync},
};

// Starting with basic string type operations
impl StringValue {
    pub fn value(&self) -> String {
        let text = self.doc.get_or_insert_text(self.property_name); // We only have one field in the yrs doc
        text.get_string(&self.doc.transact())
    }
    pub fn insert(&self, index: u32, value: &str) {
        let trx = self.record_inner.transaction_manager.handle();

        let text = self.doc.get_or_insert_text(self.property_name); // We only have one field in the yrs doc
        let mut ytx = self.doc.transact_mut();
        text.insert(&mut ytx, index, value);

        trx.add_operation(
            "yrs",
            self.record_inner.collection,
            self.record_inner.id,
            ytx.encode_update_v2(),
        );
    }
    pub fn delete(&self, index: u32, length: u32) {
        let text = self.doc.get_or_insert_text(self.property_name); // We only have one field in the yrs doc
        let mut ytx = self.doc.transact_mut();
        text.remove_range(&mut ytx, index, length);
    }
}

impl InitializeWith<String> for StringValue {
    fn initialize_with(
        inner: Arc<RecordInner>,
        property_name: &'static str,
        value: String,
    ) -> Self {
        let trx = inner.transaction_manager.handle();
        let doc = yrs::Doc::new();
        // prob an empty vec - hack until transaction mut lifetime is figured out
        let starting_state = doc.transact().state_vector();

        let text = doc.get_or_insert_text(property_name); // We only have one field in the yrs doc
                                                          // every operation in Yrs happens in scope of a transaction

        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, value.as_str());
        }
        trx.add_operation(
            "yrs",
            inner.collection,
            inner.id,
            doc.transact().state_vector().encode_v2(),
        );
        Self {
            record_inner: inner,
            property_name,
            previous_state: Arc::new(Mutex::new(starting_state)),
            doc,
        }
    }
}

impl StateSync for StringValue {
    /// Apply an update to the field from an event/operation
    fn apply_update(&self, update: &[u8]) -> Result<()> {
        let mut txn = self.doc.transact_mut();
        let update = Update::decode_v2(update)?;
        txn.apply_update(update)?;
        Ok(())
    }
    /// Retrieve the current state of the field, suitable for storing in the materialized record
    fn state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.state_vector().encode_v2()
    }
    /// Retrieve the pending update for this field since the last call to this method
    /// ideally this would be centralized in the TypeModule, rather than having to poll each field
    fn get_pending_update(&self) -> Option<Vec<u8>> {
        // hack until we figure out how to get the transaction mut to live across individual field updates
        // diff the previous state with the current state
        let mut previous_state = self.previous_state.lock().unwrap();

        let txn = self.doc.transact_mut();
        let diff = txn.encode_diff_v2(&previous_state);
        *previous_state = txn.state_vector();

        if diff.is_empty() {
            None
        } else {
            Some(diff)
        }
    }
}
