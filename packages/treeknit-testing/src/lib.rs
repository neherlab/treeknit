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

/// Depth of the trees that test the absence of recursion per tree level: a function that
/// recurses once per level overflows the stack of [`on_small_stack`] well before it.
pub const DEEP: usize = 5_000;

/// Stack size of [`on_small_stack`].
const SMALL_STACK: usize = 256 * 1024;

/// The result of `f`, run on a thread with a stack of 256 KiB, so that a test of a deep tree
/// fails the same way on every platform when the code recurses per tree level.
#[expect(clippy::expect_used, reason = "a test helper fails the test by panicking")]
pub fn on_small_stack<T: Send>(f: impl FnOnce() -> T + Send) -> T {
  std::thread::scope(|s| {
    std::thread::Builder::new()
      .stack_size(SMALL_STACK)
      .spawn_scoped(s, f)
      .expect("the test thread starts")
      .join()
      .expect("the test thread completes")
  })
}

/// The names `<prefix>0` to `<prefix><n - 1>`.
pub fn numbered(prefix: &str, n: usize) -> Vec<String> {
  (0..n).map(|i| format!("{prefix}{i}")).collect()
}

/// The Newick text of the caterpillar over `names` (at least two), without the final `;`: each
/// internal node has a leaf as its first child and the next internal node as its second, down
/// to the cherry of the last two names, as in `(L0,(L1,(L2,L3)))`.
#[expect(clippy::expect_used, reason = "a test helper fails the test by panicking")]
pub fn caterpillar_newick(names: &[String]) -> String {
  let (last, rest) = names.split_last().expect("a caterpillar has at least two leaves");
  let mut s: String = rest.iter().flat_map(|x| ["(", x.as_str(), ","]).collect();
  s.push_str(last);
  s.push_str(&")".repeat(rest.len()));
  s
}
