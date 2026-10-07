//! The serialized names of unit enum variants, such as `matched` or `seqLengths`: the names of the
//! session files, of the links, and of the TypeScript declarations, which the serde attributes of
//! each enum own.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// The name of the unit variant `value` as serde writes it.
pub fn wire_name<T: Serialize>(value: &T) -> String {
  #[expect(clippy::expect_used, reason = "unit variants always serialize")]
  serde_plain::to_string(value).expect("a unit variant serializes to its name")
}

/// The unit variant that serde reads from `text`, or `None` when no variant has that name.
pub(crate) fn from_wire_name<T: DeserializeOwned>(text: &str) -> Option<T> {
  serde_plain::from_str(text).ok()
}
