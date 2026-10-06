//! The check that a value returned to JavaScript holds only finite numbers. `serde_json` writes
//! NaN and an infinity as `null`, which the TypeScript type `number` of the field does not admit,
//! so the web app would compute with `null` instead of failing.

use serde::Serialize;
use serde::ser;
use std::fmt::{self, Display};

/// The first NaN or infinite number of `value`, with the path of its field; `Ok` when every
/// number is finite. The message of the error is `non-finite number <value>`.
pub(crate) fn check_finite<T: Serialize + ?Sized>(value: &T) -> Result<(), serde_path_to_error::Error<NonFinite>> {
  serde_path_to_error::serialize(value, FiniteCheck)
}

/// The error of [`check_finite`]: the first number that is not finite, or the message of a value
/// whose `Serialize` implementation fails by itself.
#[derive(Debug)]
pub(crate) enum NonFinite {
  Number(f64),
  Custom(String),
}

impl Display for NonFinite {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Number(value) => write!(f, "non-finite number {value}"),
      Self::Custom(message) => f.write_str(message),
    }
  }
}

impl std::error::Error for NonFinite {}

impl ser::Error for NonFinite {
  fn custom<T: Display>(message: T) -> Self {
    Self::Custom(message.to_string())
  }
}

/// A serializer that writes nothing and fails at the first number that is not finite.
#[derive(Clone, Copy)]
struct FiniteCheck;

fn finite(value: f64) -> Result<(), NonFinite> {
  if value.is_finite() {
    Ok(())
  } else {
    Err(NonFinite::Number(value))
  }
}

impl ser::Serializer for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;
  type SerializeSeq = Self;
  type SerializeTuple = Self;
  type SerializeTupleStruct = Self;
  type SerializeTupleVariant = Self;
  type SerializeMap = Self;
  type SerializeStruct = Self;
  type SerializeStructVariant = Self;

  fn serialize_f32(self, v: f32) -> Result<(), NonFinite> {
    finite(f64::from(v))
  }

  fn serialize_f64(self, v: f64) -> Result<(), NonFinite> {
    finite(v)
  }

  fn serialize_bool(self, _: bool) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_i8(self, _: i8) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_i16(self, _: i16) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_i32(self, _: i32) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_i64(self, _: i64) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_i128(self, _: i128) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_u8(self, _: u8) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_u16(self, _: u16) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_u32(self, _: u32) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_u64(self, _: u64) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_u128(self, _: u128) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_char(self, _: char) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_str(self, _: &str) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_bytes(self, _: &[u8]) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_none(self) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), NonFinite> {
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_unit_struct(self, _: &'static str) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), NonFinite> {
    Ok(())
  }

  fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, value: &T) -> Result<(), NonFinite> {
    value.serialize(self)
  }

  fn serialize_newtype_variant<T: Serialize + ?Sized>(
    self,
    _: &'static str,
    _: u32,
    _: &'static str,
    value: &T,
  ) -> Result<(), NonFinite> {
    value.serialize(self)
  }

  fn serialize_seq(self, _: Option<usize>) -> Result<Self, NonFinite> {
    Ok(self)
  }

  fn serialize_tuple(self, _: usize) -> Result<Self, NonFinite> {
    Ok(self)
  }

  fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self, NonFinite> {
    Ok(self)
  }

  fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self, NonFinite> {
    Ok(self)
  }

  fn serialize_map(self, _: Option<usize>) -> Result<Self, NonFinite> {
    Ok(self)
  }

  fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, NonFinite> {
    Ok(self)
  }

  fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self, NonFinite> {
    Ok(self)
  }
}

impl ser::SerializeSeq for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

impl ser::SerializeTuple for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

impl ser::SerializeTupleStruct for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

impl ser::SerializeTupleVariant for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

impl ser::SerializeMap for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), NonFinite> {
    key.serialize(FiniteCheck)
  }

  fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

impl ser::SerializeStruct for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

impl ser::SerializeStructVariant for FiniteCheck {
  type Ok = ();
  type Error = NonFinite;

  fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), NonFinite> {
    value.serialize(FiniteCheck)
  }

  fn end(self) -> Result<(), NonFinite> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use std::collections::BTreeMap;

  #[derive(Serialize)]
  #[serde(rename_all = "camelCase")]
  struct Outer {
    inner_value: Inner,
    items: Vec<f64>,
  }

  #[derive(Serialize)]
  struct Inner {
    x: f64,
  }

  #[derive(Serialize)]
  enum Shape {
    Circle { radius: f32 },
  }

  /// A value whose `Serialize` implementation fails by itself.
  struct Unserializable;

  impl Serialize for Unserializable {
    fn serialize<S: ser::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
      Err(ser::Error::custom("cannot serialize"))
    }
  }

  #[test]
  fn check_finite_accepts_finite_numbers() {
    let value = Outer {
      inner_value: Inner { x: f64::MAX },
      items: vec![0.0, -1.5, f64::MIN_POSITIVE],
    };
    assert_eq!(None, report(&value));
  }

  #[test]
  fn check_finite_reports_the_first_non_finite_number_with_its_path() {
    // Oracle: the camelCase names of serde and the path syntax of serde_path_to_error, which adds
    // no segment for an enum variant.
    let actual = vec![
      report(&Outer {
        inner_value: Inner { x: f64::NAN },
        items: vec![],
      }),
      report(&Outer {
        inner_value: Inner { x: 1.0 },
        items: vec![1.0, f64::INFINITY, f64::NAN],
      }),
      report(&vec![Some(1.0), Some(f64::NEG_INFINITY)]),
      report(&BTreeMap::from([("a", 1.0), ("b", f64::NAN)])),
      report(&Shape::Circle { radius: f32::NAN }),
      report(&(1.0, f32::INFINITY)),
    ];
    let expected = vec![
      Some(("innerValue.x", "non-finite number NaN")),
      Some(("items[1]", "non-finite number inf")),
      Some(("[1]", "non-finite number -inf")),
      Some(("b", "non-finite number NaN")),
      Some(("radius", "non-finite number NaN")),
      Some(("[1]", "non-finite number inf")),
    ];
    assert_eq!(
      expected
        .into_iter()
        .map(|e| e.map(|(path, message)| (path.to_owned(), message.to_owned())))
        .collect::<Vec<_>>(),
      actual
    );
  }

  #[test]
  fn check_finite_passes_on_the_error_of_a_failing_serialize() {
    let actual = report(&vec![Some(Unserializable)]);
    assert_eq!(Some(("[0]".to_owned(), "cannot serialize".to_owned())), actual);
  }

  #[test]
  fn check_finite_of_a_number_at_the_root_has_an_empty_path() {
    let error = check_finite(&f64::NAN).err();
    assert_eq!(
      Some((true, "non-finite number NaN".to_owned())),
      error.map(|e| (e.path().iter().next().is_none(), e.inner().to_string()))
    );
  }

  /// The path and the message of the first non-finite number of `value`.
  fn report<T: Serialize>(value: &T) -> Option<(String, String)> {
    check_finite(value)
      .err()
      .map(|e| (e.path().to_string(), e.inner().to_string()))
  }
}
