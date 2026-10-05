//! Run with `just test-wasm`.
#![cfg(target_arch = "wasm32")]

#[cfg(test)]
mod tests {
  use js_sys::{Error, JSON};
  use pretty_assertions::assert_eq;
  use serde_json::{Value, json};
  use treeknit_core::{Options, Resolution, Taxa, Tree};
  use treeknit_io::newick;
  use tsify::{Ts, Tsify};
  use wasm_bindgen::{JsError, JsValue};
  use wasm_bindgen_test::wasm_bindgen_test;

  #[wasm_bindgen_test]
  fn analyze_returns_plain_objects() {
    let request = json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,(D,X)));"},
            {"label": "na", "newick": "((A,(B,X)),(C,D));"},
        ],
        "settings": {"seed": 1},
    });
    let result = plain(&treeknit_wasm::analyze(&ts(&request)).unwrap().js_value());
    // Oracle: fixtures/doc_mccs_1.json (TreeKnit.jl)
    let expected_pairs = json!([{"trees": ["ha", "na"], "mccs": [["X"], ["A", "B", "C", "D"]]}]);
    assert_eq!(expected_pairs, result["pairs"]);
    assert_eq!(json!({"status": "built", "reassortments": 1}), result["arg"]);
    assert_eq!(
      json!({"name": "MCCs.dat", "mediaType": "text/plain", "text": "X\nA,B,C,D\n"}),
      result["files"][1]
    );
  }

  #[wasm_bindgen_test]
  fn default_settings_are_the_command_line_defaults() {
    let expected = json!({
        "gamma": 2, "seqLengths": null, "nMcmcIt": 50, "resolve": "matched", "preResolve": false,
        "rounds": 1, "finalRound": true, "likelihood": true, "naive": false, "seed": 1,
    });
    assert_eq!(expected, plain(&treeknit_wasm::default_settings().unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn analyze_reports_analysis_errors() {
    let one_tree = json!({"trees": [{"label": "ha", "newick": "(A,B);"}]});
    let expected = "need at least two trees";
    match treeknit_wasm::analyze(&ts(&one_tree)) {
      Ok(_) => panic!("expected error {expected:?}"),
      Err(e) => assert_eq!(expected, message(e)),
    }
  }

  #[wasm_bindgen_test]
  fn analyze_reports_where_a_request_is_malformed() {
    let expected = "invalid request: invalid type: integer `3`, expected a sequence at line 1 column 10";
    match treeknit_wasm::analyze(&ts(&json!({"trees": 3}))) {
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

  fn ts<T: Tsify>(v: &Value) -> Ts<T> {
    Ts::new_unchecked(JSON::parse(&v.to_string()).unwrap())
  }

  fn plain(v: &JsValue) -> Value {
    serde_json::from_str(&String::from(JSON::stringify(v).unwrap())).unwrap()
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
