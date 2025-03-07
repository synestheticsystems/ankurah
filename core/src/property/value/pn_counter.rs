use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::property::{
    backend::{Backends, PNBackend},
    traits::InitializeWith,
    value::ProjectedValue,
    PropertyName,
};

pub trait Integer: Copy {
    fn as_i64(self) -> i64;
    fn from_i64(i: i64) -> Self;
}

macro_rules! impl_integer {
    ($($integer:ty),*) => {
        $(
        impl Integer for $integer {
            fn as_i64(self) -> i64 {
                self as i64
            }
            fn from_i64(i: i64) -> Self {
                i as Self
            }
        }
        )*
    };
}

impl_integer!(u8, u16, u32, u64, i8, i16, i32, i64);

#[derive(Debug)]
pub struct PNCounter<I: Integer> {
    pub property_name: PropertyName,
    pub backend: Weak<PNBackend>,
    phantom: PhantomData<I>,
}

impl<I: Integer> ProjectedValue for PNCounter<I> {
    type Projected = I;
    fn projected(&self) -> Self::Projected { self.value() }
}

// Starting with basic string type operations
impl<I: Integer> PNCounter<I> {
    pub fn new(property_name: PropertyName, backend: Arc<PNBackend>) -> Self {
        Self { property_name, backend: Arc::downgrade(&backend), phantom: PhantomData }
    }
    pub fn from_backends(property_name: PropertyName, backends: &Backends) -> Self {
        let backend = backends.get::<PNBackend>().unwrap();
        Self::new(property_name, backend)
    }
    pub fn backend(&self) -> Arc<PNBackend> { self.backend.upgrade().expect("Expected `PN` property backend to exist") }
    pub fn value(&self) -> I { I::from_i64(self.backend().get(self.property_name.clone())) }
    pub fn add(&self, amount: impl Integer) { self.backend().add(self.property_name.clone(), amount.as_i64()); }
}

impl<I: Integer> InitializeWith<I> for PNCounter<I> {
    fn initialize_with(backends: &Backends, property_name: PropertyName, value: &I) -> Self {
        let new = Self::from_backends(property_name, backends);
        new.add(*value);
        new
    }
}

// TODO: Figure out whether to remove this
/*
impl StateSync for YrsString {
    // These should really be on the YrsBackend I think
    /// Apply an update to the field from an event/operation
    fn apply_update(&self, update: &[u8]) -> Result<()> {
        let yrs = self.backend();
        let mut txn = yrs.doc.transact_mut();
        let update = Update::decode_v2(update)?;
        txn.apply_update(update)?;
        Ok(())
    }
    /// Retrieve the current state of the field, suitable for storing in the materialized entity
    fn state(&self) -> Vec<u8> {
        let yrs = self.backend();
        let txn = yrs.doc.transact();
        txn.state_vector().encode_v2()
    }
    /// Retrieve the pending update for this field since the last call to this method
    /// ideally this would be centralized in the TypeModule, rather than having to poll each field
    fn get_pending_update(&self) -> Option<Vec<u8>> {
        // hack until we figure out how to get the transaction mut to live across individual field updates
        // diff the previous state with the current state
        let mut previous_state = self.previous_state.lock().unwrap();

        let yrs = self.backend();
        let txn = yrs.doc.transact_mut();
        let diff = txn.encode_diff_v2(&previous_state);
        *previous_state = txn.state_vector();

        if diff.is_empty() {
            None
        } else {
            Some(diff)
        }
    }
}
*/
