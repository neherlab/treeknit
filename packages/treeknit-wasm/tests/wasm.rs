//! Run with `just test-wasm`.
#![cfg(target_arch = "wasm32")]

#[cfg(test)]
mod tests {
  use js_sys::{Date, Error, Function, JSON};
  use pretty_assertions::assert_eq;
  use serde_json::{Value, json};
  use std::cell::{Cell, RefCell};
  use std::collections::BTreeSet;
  use std::rc::Rc;
  use treeknit_core::{Options, Resolution, Taxa, Tree};
  use treeknit_io::analysis::{self, AnalysisRequest, ParsedTrees, TreeText};
  use treeknit_io::display::{self, Scale, TreeVersion};
  use treeknit_io::figure::{self, FigureOptions, LabelMode};
  use treeknit_io::output::{self, Figure, OutputFile};
  use treeknit_io::run::{self, RunResult};
  use treeknit_wasm::Session;
  use tsify::{Ts, Tsify};
  use wasm_bindgen::prelude::Closure;
  use wasm_bindgen::{JsCast, JsValue};
  use wasm_bindgen_test::wasm_bindgen_test;

  #[wasm_bindgen_test]
  fn start_installs_the_log_capture_and_accepts_its_own_logger_again() {
    treeknit_wasm::start();
    treeknit_wasm::start();
    // Oracle: the doc of `start`: a run reports a warning only when another logger is installed.
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let summary = plain(&session.summary().unwrap().js_value());
    assert_eq!(json!([]), summary["diagnostics"]);
    assert!(session.file_text("log.txt").unwrap().contains("[INFO] TreeKnit "));
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
            "field": {"kind": "treeNewick", "index": 1},
            "message": "tree \"na\": Newick parse error: expected ',' or ')' at byte 15",
            "line": 2,
            "column": 8,
        },
        {
            "field": {"kind": "setting", "key": "gamma"},
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
    let expected = json!([{
        "field": null,
        "message": "invalid request: trees: invalid type: integer `3`, expected a sequence at line 1 column 10",
        "line": null,
        "column": null,
    }]);
    assert_eq!(
      expected,
      plain_list(&treeknit_wasm::validate(&ts(&json!({"trees": 3}))).unwrap())
    );
  }

  #[wasm_bindgen_test]
  fn session_run_throws_validation_error_for_a_malformed_request() {
    let request = json!({"trees": [{"label": "ha"}, {"label": "na", "newick": "(A,B);"}]});
    let expected = (
      "ValidationError".to_owned(),
      "invalid request: trees[0]: missing field `newick` at line 1 column 24".to_owned(),
    );
    assert_eq!(
      expected,
      thrown(Session::run(&ts(&request), &Function::new_no_args("")))
    );
  }

  #[wasm_bindgen_test]
  fn session_figure_throws_validation_error_for_malformed_options() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let options = ts(&json!({"width": "wide"}));
    let errors = vec![
      thrown(session.figure(&ts(&json!(0)), &ts(&json!("resolved")), &options)),
      thrown(session.arg_figure(&options)),
    ];
    let message = "invalid options: width: invalid type: string \"wide\", expected f64 at line 1 column 15";
    let expected = vec![("ValidationError".to_owned(), message.to_owned()); 2];
    assert_eq!(expected, errors);
  }

  #[wasm_bindgen_test]
  fn read_session_returns_the_plain_request_with_default_settings() {
    let text = r#"{"trees": [{"label": "ha", "newick": "((A,B"}], "settings": {"gamma": -1}}"#;
    let expected = json!({
        "trees": [{"label": "ha", "newick": "((A,B"}],
        "settings": {
            "gamma": -1, "seqLengths": null, "nMcmcIt": 50, "resolve": "matched", "preResolve": false,
            "rounds": 1, "finalRound": true, "likelihood": true, "naive": false, "seed": 1,
        },
    });
    assert_eq!(expected, plain(&treeknit_wasm::read_session(text).unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn read_session_loads_counts_beyond_32_bits_for_validation_to_report() {
    // Oracle: 2^32 is one above the largest usize of wasm32; the command line on a 64-bit host
    // loads it too and reports the same field error.
    let text = r#"{"trees": [], "settings": {"nMcmcIt": 4294967296}}"#;
    let request = treeknit_wasm::read_session(text).unwrap();
    let plain_request = plain(&request.js_value());
    assert_eq!(json!(1_u64 << 32), plain_request["settings"]["nMcmcIt"]);
    let errors = plain_list(&treeknit_wasm::validate(&request).unwrap());
    let expected = json!({
        "field": {"kind": "setting", "key": "nMcmcIt"},
        "message": "MCMC steps per leaf must be at most 4294967295, got 4294967296",
        "line": null,
        "column": null,
    });
    assert!(errors.as_array().unwrap().contains(&expected), "{errors}");
  }

  #[wasm_bindgen_test]
  fn read_session_throws_the_structure_error() {
    let expected = "not a TreeKnit session file: missing field `trees` at line 1 column 2";
    assert_eq!(
      ("Error".to_owned(), expected.to_owned()),
      thrown(treeknit_wasm::read_session("{}"))
    );
  }

  #[wasm_bindgen_test]
  fn session_file_returns_the_plain_session_file() {
    let request = json!({"trees": [{"label": "ha", "newick": "(A,B);"}], "settings": {"seed": 7}});
    let file = plain(&treeknit_wasm::session_file(&ts(&request)).unwrap().js_value());
    assert_eq!(
      (json!("treeknit_session.json"), json!("application/json")),
      (file["path"].clone(), file["mediaType"].clone())
    );
    let text = file["text"].as_str().unwrap();
    let back = plain(&treeknit_wasm::read_session(text).unwrap().js_value());
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
    let expected = (
      "ValidationError".to_owned(),
      "need at least two trees\ngamma must be a non-negative number, got -1".to_owned(),
    );
    assert_eq!(
      expected,
      thrown(Session::run(&ts(&request), &Function::new_no_args("")))
    );
  }

  #[wasm_bindgen_test]
  fn session_run_throws_the_error_of_the_progress_callback() {
    let calls = Rc::new(Cell::new(0));
    let counted = Rc::clone(&calls);
    let on_progress = Closure::<dyn FnMut(JsValue) -> Result<(), JsValue>>::new(move |_progress: JsValue| {
      counted.set(counted.get() + 1);
      Err(js_sys::RangeError::new("stop").into())
    });
    let (name, message) = thrown(Session::run(&ts(&two_trees()), on_progress.as_ref().unchecked_ref()));
    // Oracle: the doc of `Session.run`: the run completes without further progress calls.
    assert_eq!(
      ("RangeError".to_owned(), "stop".to_owned(), 1),
      (name, message, calls.get())
    );
  }

  #[wasm_bindgen_test]
  fn session_run_reports_plain_progress_to_done() {
    let events: Rc<RefCell<Vec<Value>>> = Rc::default();
    let sink = Rc::clone(&events);
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |p: JsValue| sink.borrow_mut().push(plain(&p)));
    Session::run(&ts(&two_trees()), callback.as_ref().unchecked_ref()).unwrap();
    let events = events.borrow();
    let keys: BTreeSet<BTreeSet<&str>> = events
      .iter()
      .map(|e| e.as_object().unwrap().keys().map(String::as_str).collect())
      .collect();
    let expected = BTreeSet::from(["fraction", "pair", "pairs", "phase", "round", "rounds"]);
    assert_eq!(BTreeSet::from([expected]), keys);
    let fractions: Vec<f64> = events.iter().map(|e| e["fraction"].as_f64().unwrap()).collect();
    // Oracle: the progress contract of `treeknit_core::Progress`: never decreasing, and only the
    // final `done` event reports 1.
    assert!(fractions.is_sorted_by(|a, b| a <= b), "{fractions:?}");
    let last = events.last().unwrap();
    assert_eq!(
      (json!("done"), Some(1.0)),
      (last["phase"].clone(), last["fraction"].as_f64())
    );
    assert_eq!(1, fractions.iter().filter(|&&f| f >= 1.0).count());
  }

  #[wasm_bindgen_test]
  fn session_summary_of_the_two_tree_example_matches_the_reference() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    // Oracle: fixtures/doc_mccs_1.json (TreeKnit.jl): MCCs [X] and [A,B,C,D], one reassortment.
    let expected = json!({
        "pairs": [{
            "index": 0, "trees": [0, 1], "labels": ["ha", "na"], "title": "ha and na", "mccCount": 2,
            "mccs": [["X"], ["A", "B", "C", "D"]], "imputedCount": 0, "ambiguousCount": 0,
        }],
        "arg": {"status": "built", "reassortments": 1},
        "noReassortment": false,
        "diagnostics": [],
    });
    assert_eq!(expected, plain(&session.summary().unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn session_summary_has_the_warnings_of_the_run() {
    let request = json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,(D,X)));\n(A,B);"},
            {"label": "na", "newick": "((A,(B,X)),(C,D):0.R);"},
        ],
    });
    let session = Session::run(&ts(&request), &Function::new_no_args("")).unwrap();
    let diagnostics = plain(&session.summary().unwrap().js_value())["diagnostics"].clone();
    // Oracle: the warning lines of `newick::ParseWarning::log`, which the command line writes.
    let expected = json!([
        {"level": "warn", "message": "ha: more than one tree in file, using the first"},
        {"level": "warn", "message": "na: ignoring invalid branch length '0.R'"},
    ]);
    let without_time: Vec<Value> = diagnostics
      .as_array()
      .unwrap()
      .iter()
      .map(|d| json!({"level": d["level"], "message": d["message"]}))
      .collect();
    assert_eq!(expected, json!(without_time));
    let time = diagnostics[0]["time"].as_str().unwrap();
    assert!(!Date::parse(time).is_nan() && time.ends_with('Z'), "{time}");
  }

  #[wasm_bindgen_test]
  fn session_run_log_leaves_out_records_of_earlier_exports() {
    let request = json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,(D,X)));\n(A,B);"},
            {"label": "na", "newick": "((A,(B,X)),(C,D));"},
        ],
    });
    treeknit_wasm::validate(&ts(&request)).unwrap();
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let summary = plain(&session.summary().unwrap().js_value());
    assert_eq!(json!([]), summary["diagnostics"]);
    assert!(!session.file_text("log.txt").unwrap().contains("more than one tree"));
  }

  #[wasm_bindgen_test]
  fn session_run_log_keeps_its_records_when_progress_calls_an_export() {
    let on_progress = Closure::<dyn FnMut(JsValue)>::new(|_progress: JsValue| {
      treeknit_wasm::inspect_tree("t", "(A,B);").unwrap();
    });
    let session = Session::run(&ts(&two_trees()), on_progress.as_ref().unchecked_ref()).unwrap();
    // Oracle: the run logs its version line before its first progress call.
    assert!(session.file_text("log.txt").unwrap().contains("[INFO] TreeKnit "));
  }

  #[wasm_bindgen_test]
  fn session_run_started_from_a_progress_callback_throws_and_the_outer_run_succeeds() {
    let nested: Rc<RefCell<Vec<String>>> = Rc::default();
    let sink = Rc::clone(&nested);
    let on_progress = Closure::<dyn FnMut(JsValue)>::new(move |_progress: JsValue| {
      let message = match Session::run(&ts(&two_trees()), &Function::new_no_args("")) {
        Ok(_) => "accepted".to_owned(),
        Err(e) => Error::from(e).message().into(),
      };
      sink.borrow_mut().push(message);
    });
    let session = Session::run(&ts(&two_trees()), on_progress.as_ref().unchecked_ref()).unwrap();
    let nested = nested.borrow();
    assert_eq!(
      BTreeSet::from(["a run is already in progress"]),
      nested.iter().map(String::as_str).collect::<BTreeSet<_>>()
    );
    assert!(session.file_text("log.txt").unwrap().contains("[INFO] TreeKnit "));
  }

  #[wasm_bindgen_test]
  fn session_run_succeeds_after_runs_that_failed() {
    let one_tree = json!({"trees": [{"label": "ha", "newick": "((A,B),(C,D));"}]});
    let invalid = Session::run(&ts(&one_tree), &Function::new_no_args(""))
      .err()
      .map(Error::from);
    let failing_progress = Closure::<dyn FnMut(JsValue) -> Result<(), JsValue>>::new(|_progress: JsValue| {
      Err(js_sys::RangeError::new("stop").into())
    });
    let stopped = Session::run(&ts(&two_trees()), failing_progress.as_ref().unchecked_ref())
      .err()
      .map(Error::from);
    assert_eq!(
      (Some("ValidationError".to_owned()), Some("RangeError".to_owned())),
      (invalid.map(|e| e.name().into()), stopped.map(|e| e.name().into()))
    );
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    assert!(session.file_text("log.txt").unwrap().contains("[INFO] TreeKnit "));
  }

  #[wasm_bindgen_test]
  fn session_files_list_every_output_with_its_size() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let files = plain_list(&session.files().unwrap());
    let paths: Vec<&str> = files
      .as_array()
      .unwrap()
      .iter()
      .map(|f| f["path"].as_str().unwrap())
      .collect();
    let expected = vec![
      "treeknit_session.json",
      "MCCs.json",
      "MCCs.dat",
      "ha_resolved.nwk",
      "na_resolved.nwk",
      "ha_imputed.nwk",
      "na_imputed.nwk",
      "auspice_ha.json",
      "auspice_na.json",
      "ARG/arg.nwk",
      "ARG/nodes.dat",
      "ARG/ha_liberal_resolved.nwk",
      "ARG/na_liberal_resolved.nwk",
      "tanglegram_ha_na.svg",
      "ARG/arg.svg",
      "parameters.json",
      "log.txt",
    ];
    assert_eq!(expected, paths);
    let sizes_match = files
      .as_array()
      .unwrap()
      .iter()
      .filter(|f| f["figure"].is_null())
      .all(|f| {
        let text = session.file_text(f["path"].as_str().unwrap()).unwrap();
        f["size"] == json!(text.len())
      });
    assert!(sizes_match, "{files}");
    assert_eq!(
      json!({"path": "MCCs.dat", "fileName": "MCCs.dat", "mediaType": "text/plain", "size": 9, "figure": null}),
      files[2]
    );
  }

  #[wasm_bindgen_test]
  fn session_figure_files_have_a_size_once_read() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let figures = |session: &Session| -> Vec<Value> {
      plain_list(&session.files().unwrap())
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| !f["figure"].is_null())
        .cloned()
        .collect()
    };
    let expected = vec![
      json!({
        "path": "tanglegram_ha_na.svg", "fileName": "tanglegram_ha_na.svg", "mediaType": "image/svg+xml", "size": null,
        "figure": {"kind": "pair", "pair": 0},
      }),
      json!({
        "path": "ARG/arg.svg", "fileName": "arg.svg", "mediaType": "image/svg+xml", "size": null,
        "figure": {"kind": "arg"},
      }),
    ];
    assert_eq!(expected, figures(&session));
    let text = session.file_text("tanglegram_ha_na.svg").unwrap();
    let sizes: Vec<Value> = figures(&session).iter().map(|f| f["size"].clone()).collect();
    assert_eq!(vec![json!(text.len()), Value::Null], sizes);
  }

  #[wasm_bindgen_test]
  fn session_figure_files_equal_the_native_figures() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let r = native_run(&two_trees());
    // Oracle: the figure files of the command line, from the same run without WebAssembly.
    let expected = (
      output::figure_text(&r, Figure::Pair { pair: 0 }).unwrap(),
      output::figure_text(&r, Figure::Arg).unwrap(),
    );
    let actual = (
      session.file_text("tanglegram_ha_na.svg").unwrap(),
      session.file_text("ARG/arg.svg").unwrap(),
    );
    assert_eq!(expected, actual);
  }

  #[wasm_bindgen_test]
  fn session_figure_draws_custom_options_and_keeps_the_default_files() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let r = native_run(&two_trees());
    let custom = FigureOptions {
      width: 640.0,
      row_height: 20.0,
      scale: Scale::Depth,
      labels: LabelMode::Off,
    };
    let view = display::pair_view(&r, 0, TreeVersion::Imputed, Scale::Depth).unwrap();
    let expected = json!({
      "fileName": "tanglegram_ha_na_imputed_depth_w640_row20_labels-off.svg",
      "text": figure::tanglegram_svg(&view, &custom).unwrap(),
    });
    let figure = session
      .figure(&ts(&json!(0)), &ts(&json!("imputed")), &ts(&json!(custom)))
      .unwrap();
    assert_eq!(expected, plain(&figure.js_value()));
    let arg = display::arg_view(&r, Scale::Depth).unwrap();
    let expected = json!({
      "fileName": "arg_depth_w640_row20_labels-off.svg",
      "text": figure::arg_svg(&arg, &custom).unwrap(),
    });
    assert_eq!(
      expected,
      plain(&session.arg_figure(&ts(&json!(custom))).unwrap().js_value())
    );
    // The figure files keep the default options after a custom figure.
    let default = output::figure_text(&r, Figure::Pair { pair: 0 }).unwrap();
    assert_eq!(default, session.file_text("tanglegram_ha_na.svg").unwrap());
    let files: Vec<OutputFile> = plain_list(&session.files().unwrap())
      .as_array()
      .unwrap()
      .iter()
      .map(|f| {
        let path = f["path"].as_str().unwrap();
        OutputFile::new(path.to_owned(), session.file_text(path).unwrap())
      })
      .collect();
    // Oracle: the archive of the listed texts; equal files give a byte-identical archive.
    assert_eq!(output::zip_archive(&files).unwrap(), session.zip().unwrap());
    assert!(
      files
        .iter()
        .any(|f| f.path == "tanglegram_ha_na.svg" && f.text == default)
    );
  }

  #[wasm_bindgen_test]
  fn session_figure_with_default_options_is_the_listed_figure_file() {
    // The example trees have no branch lengths, so the default scale `div` falls back to `depth`
    // in both.
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let pair = plain(
      &session
        .figure(&ts(&json!(0)), &ts(&json!("resolved")), &ts(&json!({})))
        .unwrap()
        .js_value(),
    );
    let arg = plain(&session.arg_figure(&ts(&json!({}))).unwrap().js_value());
    let expected = (
      json!({"fileName": "tanglegram_ha_na.svg", "text": session.file_text("tanglegram_ha_na.svg").unwrap()}),
      json!({"fileName": "arg.svg", "text": session.file_text("ARG/arg.svg").unwrap()}),
    );
    assert_eq!(expected, (pair, arg));
  }

  #[wasm_bindgen_test]
  fn session_figure_throws_validation_error_for_invalid_options() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let options = ts(&json!({"width": 0, "rowHeight": -2}));
    let errors = vec![
      thrown(session.figure(&ts(&json!(0)), &ts(&json!("resolved")), &options)),
      thrown(session.arg_figure(&options)),
    ];
    let expected = (
      "ValidationError".to_owned(),
      "figure width must be a positive number, got 0\nrow height must be a positive number, got -2".to_owned(),
    );
    assert_eq!(vec![expected.clone(), expected], errors);
  }

  #[wasm_bindgen_test]
  fn session_figure_of_an_unknown_pair_or_a_missing_arg_throws() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    assert_eq!(
      ("Error".to_owned(), "no pair 1: the run has 1 pair".to_owned()),
      thrown(session.figure(&ts(&json!(1)), &ts(&json!("resolved")), &ts(&json!({}))))
    );
    let t = "((A,B),(C,D));";
    let three =
      json!({"trees": [{"label": "ha", "newick": t}, {"label": "na", "newick": t}, {"label": "pb2", "newick": t}]});
    let session = Session::run(&ts(&three), &Function::new_no_args("")).unwrap();
    assert_eq!(
      ("Error".to_owned(), "no pair 3: the run has 3 pairs".to_owned()),
      thrown(session.figure(&ts(&json!(3)), &ts(&json!("resolved")), &ts(&json!({}))))
    );
    assert_eq!(
      (
        "Error".to_owned(),
        "the run has no ARG: it needs two trees and a built ARG".to_owned()
      ),
      thrown(session.arg_figure(&ts(&json!({}))))
    );
  }

  #[wasm_bindgen_test]
  fn session_file_texts_hold_the_request_and_the_log() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let request = plain(
      &treeknit_wasm::read_session(&session.file_text("treeknit_session.json").unwrap())
        .unwrap()
        .js_value(),
    );
    assert_eq!(
      json!(["ha", "na"]),
      json!([request["trees"][0]["label"], request["trees"][1]["label"]])
    );
    let log = session.file_text("log.txt").unwrap();
    // Oracle: the layout of the command-line log, `<time> [LEVEL] <message>`, whose first line
    // names the version.
    let (time, rest) = log.split_once(' ').unwrap();
    assert!(!Date::parse(time).is_nan(), "{log}");
    assert!(rest.starts_with("[INFO] TreeKnit "), "{log}");
    assert!(log.ends_with('\n'));
  }

  #[wasm_bindgen_test]
  fn session_file_text_of_an_unlisted_path_throws() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    assert_eq!(
      ("Error".to_owned(), "no file nope.txt".to_owned()),
      thrown(session.file_text("nope.txt"))
    );
  }

  #[wasm_bindgen_test]
  fn session_zip_holds_every_listed_file() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let bytes = session.zip().unwrap();
    assert_eq!(b"PK\x03\x04", &bytes[..4]);
    let listed = plain_list(&session.files().unwrap());
    let missing: Vec<&str> = listed
      .as_array()
      .unwrap()
      .iter()
      .map(|f| f["path"].as_str().unwrap())
      .filter(|p| {
        let name = format!("treeknit_results/{p}");
        !bytes.windows(name.len()).any(|w| w == name.as_bytes())
      })
      .collect();
    assert_eq!(Vec::<&str>::new(), missing);
  }

  #[wasm_bindgen_test]
  fn session_zip_renders_unread_figures_without_keeping_them() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let bytes = session.zip().unwrap();
    let r = native_run(&two_trees());
    let request: AnalysisRequest = serde_json::from_value(two_trees()).unwrap();
    // Oracle: the files of the command line with the web options, figures included, from the
    // same run without WebAssembly. The session file and the log are plain texts of the session.
    let native: Vec<OutputFile> = output::output_files(&r, &output::OutputOptions::web(2))
      .unwrap()
      .into_iter()
      .chain([output::parameters_file(r.options(), request.settings.seed)])
      .collect();
    let listed = plain_list(&session.files().unwrap());
    let files: Vec<OutputFile> = listed
      .as_array()
      .unwrap()
      .iter()
      .map(|f| {
        let path = f["path"].as_str().unwrap();
        native.iter().find(|n| n.path == path).cloned().unwrap_or_else(|| {
          assert!([output::SESSION_FILE, output::LOG_FILE].contains(&path), "{path}");
          OutputFile::new(path.to_owned(), session.file_text(path).unwrap())
        })
      })
      .collect();
    assert_eq!(output::zip_archive(&files).unwrap(), bytes);
    // The listing after the archive: the figures still have no size, because none was kept.
    let sizes: Vec<&Value> = listed
      .as_array()
      .unwrap()
      .iter()
      .filter(|f| !f["figure"].is_null())
      .map(|f| &f["size"])
      .collect();
    assert_eq!(vec![&Value::Null, &Value::Null], sizes);
  }

  #[wasm_bindgen_test]
  fn session_command_line_runs_the_session_file() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    assert_eq!(
      "treeknit --session treeknit_results/treeknit_session.json --outdir treeknit_results_cli --impute --auspice-view \
       --plot",
      session.command_line()
    );
  }

  #[wasm_bindgen_test]
  fn session_pair_view_of_the_two_tree_example_is_plain() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let view = plain(
      &session
        .pair_view(&ts(&json!(0)), &ts(&json!("resolved")), &ts(&json!("div")))
        .unwrap()
        .js_value(),
    );
    let leaves = |side: &str| -> Vec<String> {
      view[side]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["leaf"] == json!(true))
        .map(|n| n["name"].as_str().unwrap().to_owned())
        .collect()
    };
    assert_eq!(vec!["A", "B", "C", "D", "X"], leaves("left"));
    assert_eq!(vec!["A", "B", "X", "C", "D"], leaves("right"));
    let x = &view["left"]["nodes"]
      .as_array()
      .unwrap()
      .iter()
      .find(|n| n["name"] == json!("X"))
      .unwrap();
    assert_eq!(json!(true), x["mccBreak"]);
    assert_eq!((json!("X"), json!(1)), (x["shortName"].clone(), x["cladeSize"].clone()));
    // JSON writes whole numbers without a fraction, so they read back as integers.
    assert_eq!(json!(4), x["y"]);
    assert!(x["xDiv"].is_number() && x["xDepth"].is_number());
    assert_eq!(
      json!({
        "index": 0, "size": 1, "leaves": ["X"], "imputedLeaves": [], "ambiguousLeaves": [], "slot": 1,
        "name": "MCC 1", "rank": 1, "rows": {"first": 2, "last": 4},
      }),
      view["mccs"][0]
    );
    assert_eq!(json!({"mcc": 0, "left": [4, 4], "right": [2, 2]}), view["blocks"][2]);
    assert_eq!(json!({"left": 8, "right": 5, "mcc": 0}), view["links"][4]);
    let curve = json!({"from": [0, 4], "c1": [0.5, 4], "c2": [0.5, 2], "to": [1, 2]});
    assert_eq!(
      json!({"link": 4, "mcc": 0, "slot": 1, "curve": curve}),
      view["shapes"]["links"][4]
    );
    assert_eq!(4, view["shapes"]["ribbons"][0]["outline"].as_array().unwrap().len());
    let elbow = &view["shapes"]["left"]["elbows"][0];
    assert_eq!(
      json!(["node", "points", "mcc", "slot", "mccBreak", "added", "color"]),
      json!(keys(elbow))
    );
    assert_eq!(json!("reassortment"), view["shapes"]["left"]["marks"][0]["kind"]);
    assert_eq!(5, view["shapes"]["right"]["leaders"].as_array().unwrap().len());
    assert_eq!(
      json!(["node", "from", "to"]),
      json!(keys(&view["shapes"]["left"]["leaders"][0]))
    );
  }

  #[wasm_bindgen_test]
  fn session_pair_view_of_the_imputed_version_as_a_cladogram_is_plain() {
    let request = json!({"trees": [
        {"label": "ha", "newick": "((A,B),(C,(D,P)));"},
        {"label": "na", "newick": "((A,B),(C,D));"},
    ]});
    let session = Session::run(&ts(&request), &Function::new_no_args("")).unwrap();
    let view = plain(
      &session
        .pair_view(&ts(&json!(0)), &ts(&json!("imputed")), &ts(&json!("depth")))
        .unwrap()
        .js_value(),
    );
    let right = view["right"]["nodes"].as_array().unwrap();
    // Oracle: na imputed is ((A,B),(C,(D,P))): the root has height 3, so every leaf is at 3,
    // and P is the imputed leaf.
    let leaves: Vec<(String, Value, Value)> = right
      .iter()
      .filter(|n| n["leaf"] == json!(true))
      .map(|n| {
        (
          n["name"].as_str().unwrap().to_owned(),
          n["xDepth"].clone(),
          n["imputed"].clone(),
        )
      })
      .collect();
    let imputed: Vec<&str> = leaves
      .iter()
      .filter(|l| l.2 == json!(true))
      .map(|l| l.0.as_str())
      .collect();
    assert_eq!(vec!["P"], imputed);
    assert!(leaves.iter().all(|l| l.1 == json!(3)), "{leaves:?}");
  }

  #[wasm_bindgen_test]
  fn session_pair_view_of_an_unknown_pair_throws() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let errors = vec![
      thrown(session.pair_view(&ts(&json!(1)), &ts(&json!("resolved")), &ts(&json!("div")))),
      thrown(session.pair_view(&ts(&json!(0)), &ts(&json!("final")), &ts(&json!("div")))),
    ];
    let expected = vec![
      ("Error".to_owned(), "no pair 1: the run has 1 pair".to_owned()),
      (
        "Error".to_owned(),
        "invalid version: unknown variant `final`, expected one of `input`, `resolved`, `imputed` at line 1 column 7"
          .to_owned(),
      ),
    ];
    assert_eq!(expected, errors);
  }

  #[wasm_bindgen_test]
  fn pair_and_tree_count_reject_numbers_that_are_not_counts() {
    // Oracle: serde_json's `usize` on wasm32 (u32) rejects a fraction, a negative number, a
    // number above u32::MAX, and `null`, which `JSON.stringify` writes for NaN.
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let numbers = [1.5, -1.0, 4_294_967_296.0, f64::NAN];
    let pair_errors: Vec<(String, String)> = numbers
      .iter()
      .map(|&x| {
        let pair = Ts::new_unchecked(JsValue::from_f64(x));
        thrown(session.pair_view(&pair, &ts(&json!("input")), &ts(&json!("div"))))
      })
      .collect();
    let count_errors: Vec<(String, String)> = numbers
      .iter()
      .map(|&x| {
        thrown(treeknit_wasm::settings_schema(
          &Ts::new_unchecked(JsValue::from_f64(x)),
          &ts(&json!({})),
        ))
      })
      .collect();
    let rejections = [
      "invalid type: floating point `1.5`, expected usize at line 1 column 3",
      "invalid value: integer `-1`, expected usize at line 1 column 2",
      "invalid value: integer `4294967296`, expected usize at line 1 column 10",
      "invalid type: null, expected usize at line 1 column 4",
    ];
    let expected = (
      rejections
        .map(|r| ("Error".to_owned(), format!("invalid pair: {r}")))
        .to_vec(),
      rejections
        .map(|r| ("Error".to_owned(), format!("invalid k: {r}")))
        .to_vec(),
    );
    assert_eq!(expected, (pair_errors, count_errors));
  }

  #[wasm_bindgen_test]
  fn session_auspice_view_of_the_two_tree_example_has_the_leaves_of_the_pair_view() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let (version, scale) = (ts(&json!("resolved")), ts(&json!("div")));
    let pair = plain(&session.pair_view(&ts(&json!(0)), &version, &scale).unwrap().js_value());
    let auspice = plain(
      &session
        .auspice_view(&ts(&json!(0)), &version, &scale)
        .unwrap()
        .js_value(),
    );
    let (expected, actual): (Vec<_>, Vec<_>) = ["left", "right"]
      .into_iter()
      .map(|side| {
        let drawn: Vec<&Value> = pair[side]["nodes"]
          .as_array()
          .unwrap()
          .iter()
          .filter(|n| n["leaf"] == json!(true))
          .map(|n| &n["name"])
          .collect();
        let tree = &auspice[side];
        (
          (side, drawn, json!("v2"), json!("mcc")),
          (
            side,
            preorder_leaves(&tree["tree"]),
            tree["version"].clone(),
            tree["meta"]["display_defaults"]["color_by"].clone(),
          ),
        )
      })
      .unzip();
    assert_eq!(expected, actual);
  }

  /// The leaf names of the nested Auspice tree `tree` in preorder.
  fn preorder_leaves(tree: &Value) -> Vec<&Value> {
    let mut leaves = Vec::new();
    let mut stack = vec![tree];
    while let Some(n) = stack.pop() {
      match n.get("children") {
        Some(children) => stack.extend(children.as_array().unwrap().iter().rev()),
        None => leaves.push(&n["name"]),
      }
    }
    leaves
  }

  #[wasm_bindgen_test]
  fn session_auspice_view_of_an_unknown_pair_throws() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    assert_eq!(
      ("Error".to_owned(), "no pair 1: the run has 1 pair".to_owned()),
      thrown(session.auspice_view(&ts(&json!(1)), &ts(&json!("resolved")), &ts(&json!("div"))))
    );
  }

  #[wasm_bindgen_test]
  fn session_auspice_files_hold_the_datasets_of_the_auspice_view() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let (version, scale) = (ts(&json!("imputed")), ts(&json!("depth")));
    let auspice = plain(
      &session
        .auspice_view(&ts(&json!(0)), &version, &scale)
        .unwrap()
        .js_value(),
    );
    let files = plain(
      &session
        .auspice_files(&ts(&json!(0)), &version, &scale, &ts(&json!("right")))
        .unwrap()
        .js_value(),
    );
    let labels = plain(&session.summary().unwrap().js_value())["pairs"][0]["labels"].clone();
    let stem = format!(
      "auspice_{}_{}_imputed_depth",
      labels[0].as_str().unwrap(),
      labels[1].as_str().unwrap()
    );
    let json = files["json"].as_array().unwrap();
    assert_eq!(
      json!(format!("{stem}_{}", labels[1].as_str().unwrap())),
      files["svgPrefix"]
    );
    assert_eq!(1, json.len());
    assert_eq!(
      json!(format!("{stem}_{}.json", labels[1].as_str().unwrap())),
      json[0]["path"]
    );
    let text: Value = serde_json::from_str(json[0]["text"].as_str().unwrap()).unwrap();
    // A JavaScript round trip writes whole numbers without a fraction, so numbers compare as f64.
    assert_eq!(numbers_as_f64(&auspice["right"]), numbers_as_f64(&text));
  }

  /// `value` with every number as an `f64`, so values that differ only in the integer or float
  /// form of their numbers compare equal.
  fn numbers_as_f64(value: &Value) -> Value {
    match value {
      Value::Number(n) => json!(n.as_f64().unwrap()),
      Value::Array(items) => Value::Array(items.iter().map(numbers_as_f64).collect()),
      Value::Object(fields) => Value::Object(fields.iter().map(|(k, v)| (k.clone(), numbers_as_f64(v))).collect()),
      other => other.clone(),
    }
  }

  #[wasm_bindgen_test]
  fn session_arg_view_of_the_two_tree_example_is_plain() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let view = plain(&session.arg_view(&ts(&json!("depth"))).unwrap().unwrap().js_value());
    assert_eq!(json!("shared"), view["rootCase"]);
    let nodes = view["nodes"].as_array().unwrap();
    let root = &nodes[usize::try_from(view["root"].as_u64().unwrap()).unwrap()];
    assert_eq!(json!([null, null]), root["parents"]);
    assert_eq!(1, nodes.iter().filter(|n| n["hybrid"] == json!(true)).count());
    let kinds: BTreeSet<&str> = view["shapes"]["edges"]
      .as_array()
      .unwrap()
      .iter()
      .map(|e| e["path"]["kind"].as_str().unwrap())
      .collect();
    assert_eq!(BTreeSet::from(["curve", "elbow"]), kinds);
    assert_eq!(json!("hybrid"), view["shapes"]["marks"][0]["kind"]);
    assert_eq!(
      json!([
        "label",
        "shortLabel",
        "parents",
        "children",
        "tau",
        "hybrid",
        "leaf",
        "segments",
        "xDiv",
        "xDepth",
        "y",
        "rows"
      ]),
      json!(keys(root))
    );
  }

  #[wasm_bindgen_test]
  fn session_arg_view_of_three_trees_is_undefined() {
    let t = "((A,B),(C,D));";
    let request =
      json!({"trees": [{"label": "a", "newick": t}, {"label": "b", "newick": t}, {"label": "c", "newick": t}]});
    let session = Session::run(&ts(&request), &Function::new_no_args("")).unwrap();
    assert!(session.arg_view(&ts(&json!("div"))).unwrap().is_none());
  }

  #[wasm_bindgen_test]
  fn session_constellation_of_the_two_tree_example_is_plain() {
    let session = Session::run(&ts(&two_trees()), &Function::new_no_args("")).unwrap();
    let table = plain(&session.constellation().unwrap().js_value());
    assert_eq!(json!(["A", "B", "C", "D", "X"]), table["leaves"]);
    assert_eq!(json!(["ha and na"]), table["pairTitles"]);
    assert_eq!(json!(["MCC 1", "MCC 2"]), table["mccNames"]);
    assert_eq!(json!([{"mcc": 0, "size": 1, "slot": 1}]), table["cells"][4]);
    assert_eq!(json!([{"mcc": 1, "size": 4, "slot": 0}]), table["cells"][0]);
  }

  #[wasm_bindgen_test]
  fn drawing_rules_are_plain() {
    let expected = json!({
      "labelAutoMinRowPx": 10, "linkMinRowPx": 6, "labelMaxChars": 40, "labelFontPx": 12, "legendSymbolPx": 24, "marginPx": 16, "labelGapPx": 6,
      "linkZoneShare": 0.2, "linkZoneMinShare": 0.15, "tanglegramLabelColumnMaxShare": 0.25,
      "argLabelColumnMaxShare": 0.25, "branchWidthPx": 1.5, "reassortmentWidthPx": 2, "linkWidthPx": 1,
      "leaderWidthPx": 1, "leaderOpacity": 0.5, "markRadiusPx": 3.5, "markLinePx": 1.5, "ribbonOpacity": 0.55,
      "dashPx": [4, 3], "dotPx": [1, 3],
    });
    assert_eq!(expected, plain(&treeknit_wasm::drawing_rules().unwrap().js_value()));
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
    let expected = "invalid trees[1]: label: invalid type: integer `3`, expected a string at line 1 column 10";
    assert_eq!(
      ("Error".to_owned(), expected.to_owned()),
      thrown(treeknit_wasm::overlap(trees))
    );
  }

  #[wasm_bindgen_test]
  fn settings_schema_returns_plain_rules_for_the_settings() {
    let schema = treeknit_wasm::settings_schema(&ts(&json!(2)), &ts(&json!({"naive": true}))).unwrap();
    let schema = plain(&schema.js_value());
    let expected = json!({
        "default": 2, "min": 0, "minExclusive": false, "max": null, "step": null, "integer": false,
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
    let expected = "invalid settings: foo: unknown field `foo`, expected one of `gamma`, `seqLengths`, `nMcmcIt`, \
                    `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed` at line 1 column 6";
    assert_eq!(
      ("Error".to_owned(), expected.to_owned()),
      thrown(treeknit_wasm::settings_schema(&ts(&json!(2)), &ts(&json!({"foo": 1}))))
    );
  }

  #[wasm_bindgen_test]
  fn version_follows_the_command_line_rule() {
    // Oracle: the rule of `packages/version_rule.rs`, the version of `treeknit --version`.
    let version = option_env!("TREEKNIT_VERSION")
      .filter(|v| !v.is_empty())
      .map_or_else(|| format!("{}-dev", env!("CARGO_PKG_VERSION")), str::to_owned);
    let expected = json!({
      "version": version,
      "repository": "https://github.com/neherlab/treeknit-rs",
      "releases": "https://github.com/neherlab/treeknit-rs/releases",
      "newIssue": "https://github.com/neherlab/treeknit-rs/issues/new",
    });
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

  /// The two-tree example: X moved between the trees.
  #[wasm_bindgen_test]
  fn parse_launch_returns_the_launch_and_the_ignored_keys() {
    let entries = [
      ("example", "5-leaves"),
      ("gamma", "3"),
      ("run", ""),
      ("view", "mccs"),
      ("gama", "1"),
    ]
    .map(|(key, value)| ts(&json!({"key": key, "value": value})))
    .to_vec();
    let parsed = treeknit_wasm::parse_launch(entries, vec!["view".to_owned()]).unwrap();
    let expected = json!({
        "launch": {
            "input": {"kind": "example", "id": "5-leaves"},
            "settings": {
                "gamma": 3, "seqLengths": null, "nMcmcIt": null, "resolve": null, "preResolve": null,
                "rounds": null, "finalRound": null, "likelihood": null, "naive": null, "seed": null,
            },
            "run": true,
        },
        "errors": [],
        "ignored": [{"key": "gama", "suggestion": "gamma", "afterLocationQuery": false}],
    });
    assert_eq!(expected, plain(&parsed.js_value()));
  }

  #[wasm_bindgen_test]
  fn apply_settings_replaces_the_values_of_the_patch() {
    let base = json!({"gamma": 3, "seed": 9});
    let patch = json!({
        "gamma": null, "seqLengths": null, "nMcmcIt": null, "resolve": "strict", "preResolve": null,
        "rounds": null, "finalRound": false, "likelihood": null, "naive": null, "seed": 7,
    });
    let settings = plain(
      &treeknit_wasm::apply_settings(&ts(&base), &ts(&patch))
        .unwrap()
        .js_value(),
    );
    let expected = json!([3, "strict", false, 7]);
    assert_eq!(
      expected,
      json!([
        settings["gamma"],
        settings["resolve"],
        settings["finalRound"],
        settings["seed"]
      ])
    );
  }

  #[wasm_bindgen_test]
  fn decode_tree_bytes_rejects_a_web_page() {
    assert_eq!(
      ("Error".to_owned(), "a web page, not a tree file".to_owned()),
      thrown(treeknit_wasm::decode_tree_bytes(b"<!doctype html><p>404</p>"))
    );
    assert_eq!("(A,B);", treeknit_wasm::decode_tree_bytes(b"(A,B);").unwrap());
  }

  #[wasm_bindgen_test]
  fn examples_list_the_catalog() {
    let examples = plain_list(&treeknit_wasm::examples().unwrap());
    let small = json!({
        "id": "5-leaves", "name": "5 leaves", "group": "small", "groupName": "Small",
        "trees": [
            {"file": "ha.nwk", "label": "ha", "path": null, "newick": "((A,B),(C,(D,X)));"},
            {"file": "na.nwk", "label": "na", "path": null, "newick": "((A,(B,X)),(C,D));"},
        ],
    });
    assert_eq!(small, examples[0]);
  }

  #[wasm_bindgen_test]
  fn launch_pairs_write_the_example_and_its_inline_session_reads_back() {
    let source = json!({
        "request": two_trees(),
        "addresses": [
            {"kind": "example", "id": "5-leaves", "file": "ha.nwk"},
            {"kind": "example", "id": "5-leaves", "file": "na.nwk"},
        ],
        "run": true,
    });
    let pairs = treeknit_wasm::launch_pairs(&ts(&source)).unwrap().unwrap();
    assert_eq!(
      json!([{"key": "example", "value": "5-leaves"}, {"key": "run", "value": ""}]),
      plain_list(&pairs)
    );
    let unaddressed = json!({"request": two_trees(), "addresses": [null, null], "run": false});
    assert!(treeknit_wasm::launch_pairs(&ts(&unaddressed)).unwrap().is_none());
    let inline = treeknit_wasm::inline_session(&ts(&two_trees())).unwrap();
    let entries = vec![ts(&json!({"key": "session", "value": inline}))];
    let parsed = plain(&treeknit_wasm::parse_launch(entries, vec![]).unwrap().js_value());
    let text = parsed["launch"]["input"]["location"]["text"].as_str().unwrap();
    let request = plain(&treeknit_wasm::read_session(text).unwrap().js_value());
    assert_eq!(two_trees()["trees"], request["trees"]);
  }

  #[wasm_bindgen_test]
  fn link_limits_are_those_of_the_command_line() {
    let expected = json!({
        "fetchTimeoutSeconds": 60, "maxDownloadBytes": 64 * 1024 * 1024, "maxLinkChars": 32_000, "longLinkChars": 2_000,
    });
    assert_eq!(expected, plain(&treeknit_wasm::link_limits().unwrap().js_value()));
  }

  #[wasm_bindgen_test]
  fn result_names_are_those_of_the_command_line() {
    // Oracle: the default `--outdir` of the command line, zipped, and its file paths.
    let expected = json!({
        "archive": "treeknit_results.zip", "archiveMediaType": "application/zip", "resultsDir": "treeknit_results",
        "commandLineResultsDir": "treeknit_results_cli", "argNewick": "ARG/arg.nwk",
        "auspiceFiles": ["auspice_ha.json", "auspice_na.json"],
    });
    assert_eq!(
      expected,
      plain(&treeknit_wasm::result_names("ha", "na").unwrap().js_value())
    );
  }

  #[wasm_bindgen_test]
  fn url_place_names_the_host_and_the_decoded_file_name() {
    // Oracle: the host without the port, and the last path segment with %20 decoded.
    assert_eq!(
      json!({"host": "x.org", "fileName": "seg 4.nwk"}),
      plain(
        &treeknit_wasm::url_place("https://x.org:8443/a/seg%204.nwk?raw=1")
          .unwrap()
          .js_value()
      )
    );
  }

  fn two_trees() -> Value {
    json!({
        "trees": [
            {"label": "ha", "newick": "((A,B),(C,(D,X)));"},
            {"label": "na", "newick": "((A,(B,X)),(C,D));"},
        ],
    })
  }

  /// The run of `request`, as `Session::run` builds it, without WebAssembly
  /// bindings.
  fn native_run(request: &Value) -> RunResult {
    let request: AnalysisRequest = serde_json::from_value(request.clone()).unwrap();
    let opts = analysis::options(&request.settings, request.trees.len(), false).unwrap();
    let parsed = analysis::parse_trees(&request.trees).unwrap();
    run::run(parsed, &opts, request.settings.seed, &|_| {})
  }

  /// `v` as an argument of an export. Every test passes its arguments through here, so it also
  /// installs the log capture, which the start function installs in the web app but
  /// wasm-bindgen-test does not run.
  fn ts<T: Tsify>(v: &Value) -> Ts<T> {
    treeknit_wasm::start();
    Ts::new_unchecked(JSON::parse(&v.to_string()).unwrap())
  }

  fn plain(v: &JsValue) -> Value {
    serde_json::from_str(&String::from(JSON::stringify(v).unwrap())).unwrap()
  }

  fn plain_list<T: Tsify>(values: &[Ts<T>]) -> Value {
    Value::Array(values.iter().map(|v| plain(&v.js_value())).collect())
  }

  /// The keys of the JSON object `v`, in their order.
  fn keys(v: &Value) -> Vec<&str> {
    v.as_object().unwrap().keys().map(String::as_str).collect()
  }

  /// The name and the message of the JavaScript error that `result` throws.
  fn thrown<T>(result: Result<T, impl Into<JsValue>>) -> (String, String) {
    let Err(e) = result else {
      panic!("expected a thrown error");
    };
    let error: Error = e.into().unchecked_into();
    (String::from(error.name()), String::from(error.message()))
  }

  fn parsed(newicks: &[&str]) -> (Vec<Tree>, Taxa) {
    let texts: Vec<TreeText> = newicks
      .iter()
      .enumerate()
      .map(|(i, s)| TreeText::new(format!("t{i}"), *s))
      .collect();
    let ParsedTrees { trees, taxa } = analysis::parse_trees(&texts).unwrap();
    (trees, taxa)
  }
}
