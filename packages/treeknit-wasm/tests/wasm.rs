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
  fn read_request_returns_the_plain_request_with_default_settings() {
    let text = r#"{"trees": [{"label": "ha", "newick": "((A,B"}], "settings": {"gamma": -1}}"#;
    let expected = json!({
        "trees": [{"label": "ha", "newick": "((A,B"}],
        "settings": {
            "gamma": -1, "seqLengths": null, "nMcmcIt": 50, "resolve": "matched", "preResolve": false,
            "rounds": 1, "finalRound": true, "likelihood": true, "naive": false, "seed": 1,
        },
    });
    assert_eq!(expected, plain(&treeknit_wasm::read_request(text).unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn read_request_throws_the_structure_error() {
    let expected = "not a TreeKnit session file: missing field `trees` at line 1 column 2";
    match treeknit_wasm::read_request("{}") {
      Ok(_) => panic!("expected error {expected:?}"),
      Err(e) => assert_eq!(expected, message(e)),
    }
  }

  #[wasm_bindgen_test]
  fn request_file_returns_the_plain_session_file() {
    let request = json!({"trees": [{"label": "ha", "newick": "(A,B);"}], "settings": {"seed": 7}});
    let file = plain(&treeknit_wasm::request_file(&ts(&request)).unwrap().js_value());
    assert_eq!(
      (json!("treeknit_request.json"), json!("application/json")),
      (file["path"].clone(), file["mediaType"].clone())
    );
    let text = file["text"].as_str().unwrap();
    let back = plain(&treeknit_wasm::read_request(text).unwrap().js_value());
    assert_eq!(
      (json!("ha"), json!(7)),
      (back["trees"][0]["label"].clone(), back["settings"]["seed"].clone())
    );
  }

  #[wasm_bindgen_test]
  fn tree_labels_follow_the_web_label_policy() {
    let names = vec!["ha.nwk".to_owned(), "ha.tree".to_owned()];
    let existing = vec!["HA".to_owned()];
    assert_eq!(vec!["ha_2", "ha_3"], treeknit_wasm::tree_labels(names, existing));
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
  }

  #[wasm_bindgen_test]
  fn inspect_tree_returns_a_plain_inspection() {
    let expected = json!({
        "label": "ha", "leaves": 3, "internalNodes": 2, "polytomies": 0, "branchLengths": "some",
        "warnings": ["more than one tree in file, using the first"], "error": null,
    });
    let actual = treeknit_wasm::inspect_tree("ha", "((A:1,B:1),C);\n(A,B,C);").unwrap();
    assert_eq!(expected, plain(&actual.js_value()));
  }

  #[wasm_bindgen_test]
  fn inspect_tree_returns_the_parse_error_with_its_position() {
    let expected = json!({"message": "expected ',' or ')'", "line": 2, "column": 8});
    let actual = treeknit_wasm::inspect_tree("na", "((A,B),\n(C,D)x y);").unwrap();
    assert_eq!(expected, plain(&actual.js_value())["error"]);
  }

  #[wasm_bindgen_test]
  fn overlap_returns_plain_trees_pairs_and_failures() {
    let trees = [
      json!({"label": "ha", "newick": "((A,B),(C,D));"}),
      json!({"label": "broken", "newick": "((A,B)"}),
      json!({"label": "na", "newick": "(A,(E,F));"}),
    ];
    let expected = json!({
        "totalLeaves": 6,
        "trees": [
            {"index": 0, "label": "ha", "leaves": 4, "missing": 2},
            {"index": 2, "label": "na", "leaves": 3, "missing": 3},
        ],
        "pairs": [{"i": 0, "j": 2, "shared": 1, "blocked": true}],
        "failed": [1],
    });
    let actual = treeknit_wasm::overlap(trees.iter().map(ts).collect()).unwrap();
    assert_eq!(expected, plain(&actual.js_value()));
  }

  #[wasm_bindgen_test]
  fn overlap_names_the_malformed_tree() {
    let trees = vec![
      ts(&json!({"label": "ha", "newick": "(A,B);"})),
      ts(&json!({"label": 3, "newick": "(A,B);"})),
    ];
    let expected = "invalid trees[1]: invalid type: integer `3`, expected a string at line 1 column 10";
    match treeknit_wasm::overlap(trees) {
      Ok(_) => panic!("expected error {expected:?}"),
      Err(e) => assert_eq!(expected, message(e)),
    }
  }

  #[wasm_bindgen_test]
  fn settings_schema_returns_plain_rules_for_the_settings() {
    let schema = treeknit_wasm::settings_schema(2, &ts(&json!({"naive": true}))).unwrap();
    let schema = plain(&schema.js_value());
    let expected = json!({
        "default": 2, "min": 0, "minExclusive": false, "max": null, "step": 0.1, "integer": false,
        "applies": false, "reason": "Naive MCCs skip the inference that uses this setting.",
        "help": "Cost γ of a reassortment, that is of removing an MCC.",
    });
    assert_eq!(expected, schema["settings"]["gamma"]);
    assert_eq!(
      json!(["matched", "strict", "liberal", "none"]),
      json!(
        schema["modes"]
          .as_array()
          .unwrap()
          .iter()
          .map(|m| m["mode"].clone())
          .collect::<Vec<_>>()
      )
    );
  }

  #[wasm_bindgen_test]
  fn settings_schema_names_malformed_settings() {
    let expected = "invalid settings: unknown field `foo`";
    match treeknit_wasm::settings_schema(2, &ts(&json!({"foo": 1}))) {
      Ok(_) => panic!("expected error {expected:?}"),
      Err(e) => assert!(message(e).starts_with(expected)),
    }
  }

  #[wasm_bindgen_test]
  fn version_follows_the_command_line_rule() {
    // Oracle: the rule of `packages/version_rule.rs`, the version of `treeknit --version`.
    let version = option_env!("TREEKNIT_VERSION")
      .filter(|v| !v.is_empty())
      .map_or_else(|| format!("{}-dev", env!("CARGO_PKG_VERSION")), str::to_owned);
    let expected = json!({"version": version, "repository": "https://github.com/neherlab/treeknit-rs"});
    assert_eq!(expected, plain(&treeknit_wasm::version().unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn palette_returns_the_plain_palette() {
    let expected = serde_json::to_value(treeknit_io::palette::palette()).unwrap();
    let actual = plain(&treeknit_wasm::palette().unwrap().js_value());
    assert_eq!(expected, actual);
    assert_eq!(json!("#2f4b9a"), actual["light"]["mcc"][0]);
    assert_eq!(json!("#83908d"), actual["light"]["noMcc"]);
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
