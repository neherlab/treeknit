//! Test helpers shared by the TreeKnit crates.

#[doc(hidden)]
pub use pretty_assertions;

/// Assert that `result` is an error whose text is exactly `expected`. The text is `{error:#}`:
/// the whole chain of an `eyre::Report`, joined by `: `, and the message of any other error.
#[macro_export]
macro_rules! assert_err {
  ($result:expr, $expected:expr $(,)?) => {{
    let Err(error) = $result else {
      panic!("expected Err, got Ok");
    };
    $crate::pretty_assertions::assert_eq!($expected, format!("{error:#}"));
  }};
}
