//! Run with `just test-wasm`.
#![cfg(target_arch = "wasm32")]

#[cfg(test)]
mod tests {
  use js_sys::{Error, Function, JSON};
  use pretty_assertions::assert_eq;
  use serde_json::{Value, json};
  use treeknit_core::{Options, Resolution, Taxa, Tree};
  use treeknit_io::newick;
  use treeknit_wasm::Session;
  use tsify::{Ts, Tsify};
  use wasm_bindgen::{JsError, JsValue};
  use wasm_bindgen_test::wasm_bindgen_test;

  #[wasm_bindgen_test]
  fn default_settings_are_the_command_line_defaults() {
    let expected = json!({
        "gamma": 2, "seqLengths": null, "nMcmcIt": 50, "resolve": "matched", "preResolve": false,
        "rounds": 1, "finalRound": true, "likelihood": true, "naive": false, "seed": 1,
    });
    assert_eq!(expected, plain(&treeknit_wasm::default_settings().unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn validate_returns_plain_errors() {
    let request = json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,D));"},
            {"label": "na", "newick": "((A,B),\n(C,D)x y);"},
        ],
        "settings": {"gamma": -1},
    });
    let expected = json!([
        {
            "field": "trees[1].newick",
            "message": "tree \"na\": Newick parse error: expected ',' or ')' at byte 15",
            "line": 2,
            "column": 8,
        },
        {
            "field": "settings.gamma",
            "message": "gamma must be a non-negative number, got -1",
            "line": null,
            "column": null,
        },
    ]);
    assert_eq!(expected, plain_list(&treeknit_wasm::validate(&ts(&request)).unwrap()));
  }

  #[wasm_bindgen_test]
  fn validate_accepts_a_valid_request() {
    let request = json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,(D,X)));"},
            {"label": "na", "newick": "((A,(B,X)),(C,D));"},
        ],
    });
    assert_eq!(json!([]), plain_list(&treeknit_wasm::validate(&ts(&request)).unwrap()));
  }

  #[wasm_bindgen_test]
  fn validate_reports_where_a_request_is_malformed() {
    let expected = "invalid request: invalid type: integer `3`, expected a sequence at line 1 column 10";
    match treeknit_wasm::validate(&ts(&json!({"trees": 3}))) {
      Ok(_) => panic!("expected error {expected:?}"),
      Err(e) => assert_eq!(expected, message(e)),
    }
  }

  #[wasm_bindgen_test]
  fn core_parallel_pairs_run_on_the_calling_thread() {
    // Without threads, rayon falls back to the calling thread. Pairs run in parallel only
    // without resolution.
    let three = ["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "(((A,B),C),(D,X));"];
    let mccs = |parallel: bool| {
      let (mut trees, taxa) = parsed(&three);
      let opts = Options {
        parallel,
        resolution: Resolution::None,
        ..Options::for_trees(3)
      };
      treeknit_core::run(&mut trees, &taxa, &opts, 1)
        .into_iter()
        .map(|p| p.mccs)
        .collect::<Vec<_>>()
    };
    assert_eq!(mccs(false), mccs(true));
  }

  #[wasm_bindgen_test]
  fn session_run_throws_validation_error_for_an_invalid_request() {
    let request = json!({
        "trees": [{"label": "ha", "newick": "((A,B),(C,D));"}],
        "settings": {"gamma": -1},
    });
    let error = match Session::run(&ts(&request), &Function::new_no_args("")) {
      Ok(_) => panic!("expected a ValidationError"),
      Err(e) => Error::from(e),
    };
    assert_eq!("ValidationError", String::from(error.name()));
    assert_eq!(
      "need at least two trees\ngamma must be a non-negative number, got -1",
      String::from(error.message())
    );
  }

  #[wasm_bindgen_test]
  fn not_built_exports_throw_not_implemented() {
    let request = json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,(D,X)));"},
            {"label": "na", "newick": "((A,(B,X)),(C,D));"},
        ],
    });
    let error = match Session::run(&ts(&request), &Function::new_no_args("")) {
      Ok(_) => panic!("expected an error"),
      Err(e) => Error::from(e),
    };
    assert_eq!("Error", String::from(error.name()));
    assert_eq!("not implemented: Session.run", String::from(error.message()));
    match treeknit_wasm::inspect_tree("ha", "(A,B);") {
      Ok(_) => panic!("expected an error"),
      Err(e) => assert_eq!("not implemented: inspectTree", message(e)),
    }
  }

  fn ts<T: Tsify>(v: &Value) -> Ts<T> {
    Ts::new_unchecked(JSON::parse(&v.to_string()).unwrap())
  }

  fn plain(v: &JsValue) -> Value {
    serde_json::from_str(&String::from(JSON::stringify(v).unwrap())).unwrap()
  }

  fn plain_list<T: Tsify>(values: &[Ts<T>]) -> Value {
    Value::Array(values.iter().map(|v| plain(&v.js_value())).collect())
  }

  fn message(e: JsError) -> String {
    Error::from(JsValue::from(e)).message().into()
  }

  fn parsed(newicks: &[&str]) -> (Vec<Tree>, Taxa) {
    let mut trees: Vec<Tree> = newicks
      .iter()
      .enumerate()
      .map(|(i, s)| newick::parse(s, &format!("t{i}")).unwrap())
      .collect();
    let taxa = Taxa::from_trees(&trees);
    for t in &mut trees {
      t.assign_taxa(&taxa).unwrap();
    }
    (trees, taxa)
  }
}
