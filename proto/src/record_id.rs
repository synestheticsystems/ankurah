use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;
#[derive(PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ID(pub Ulid);

impl fmt::Debug for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let humanized = crate::human_id::hex(self.0.to_bytes());
        f.debug_tuple("ID").field(&humanized).finish()
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let humanized = crate::human_id::hex(self.0.to_bytes());
        f.debug_tuple("ID").field(&humanized).finish()
    }
}

impl AsRef<ID> for ID {
    fn as_ref(&self) -> &ID {
        self
    }
}
