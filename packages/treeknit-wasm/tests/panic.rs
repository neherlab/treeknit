//! Run with `just test-wasm`. A test binary of its own, because a panic leaves the WebAssembly
//! instance unusable for the tests that follow it.
#![cfg(target_arch = "wasm32")]

#[cfg(test)]
mod tests {
  use js_sys::{Error, Function, WebAssembly};
  use pretty_assertions::assert_eq;
  use std::cell::RefCell;
  use std::rc::Rc;
  use wasm_bindgen::prelude::Closure;
  use wasm_bindgen::{JsCast, JsValue};
  use wasm_bindgen_test::wasm_bindgen_test;

  #[wasm_bindgen_test]
  fn panic_text_reaches_the_sink_for_each_panic() {
    treeknit_wasm::start();
    let texts: Rc<RefCell<Vec<String>>> = Rc::default();
    let received = Rc::clone(&texts);
    let sink = Closure::<dyn FnMut(String)>::new(move |text: String| received.borrow_mut().push(text));
    treeknit_wasm::set_panic_sink(sink.as_ref().unchecked_ref());
    let first = Closure::<dyn FnMut()>::new(|| panic!("first failure"));
    let second = Closure::<dyn FnMut()>::new(|| panic!("second failure"));
    let errors: Vec<(bool, String)> = [&first, &second]
      .into_iter()
      .map(|panicking| {
        let function: &Function = panicking.as_ref().unchecked_ref();
        match function.call0(&JsValue::UNDEFINED) {
          Ok(_) => (false, "returned".to_owned()),
          Err(e) => (
            e.is_instance_of::<WebAssembly::RuntimeError>(),
            Error::from(e).name().into(),
          ),
        }
      })
      .collect();
    // Oracle: the doc of `set_panic_sink`: each panic passes its text, with its location, to the
    // sink, and the call then throws a `WebAssembly.RuntimeError`.
    assert_eq!(
      vec![(true, "RuntimeError".to_owned()), (true, "RuntimeError".to_owned())],
      errors
    );
    // Oracle: the `Display` of `std::panic::PanicHookInfo`: `panicked at <location>:`, a line
    // break, then the message.
    let texts: Vec<(bool, Option<String>)> = texts
      .borrow()
      .iter()
      .map(|t| {
        (
          t.starts_with("panicked at "),
          t.split_once('\n').map(|(_, message)| message.to_owned()),
        )
      })
      .collect();
    assert_eq!(
      vec![
        (true, Some("first failure".to_owned())),
        (true, Some("second failure".to_owned()))
      ],
      texts
    );
  }
}
