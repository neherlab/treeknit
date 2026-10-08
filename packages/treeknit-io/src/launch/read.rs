//! Reading the key-value pairs of a link into a launch.

use crate::analysis::{self, Field, MIN_TREES, ResolveMode, ValidationError};
use crate::examples;
use crate::launch::{
  FORMAT_VERSION, IgnoredKey, Launch, LaunchInput, LaunchParse, LaunchTree, LinkLocation, MessageSource, SettingsPatch,
  parse_location, split_label,
};
use crate::schema::{KeyValue, SETTING_KEYS, SettingKey, SettingName};
use crate::wire::from_wire_name;
use std::collections::BTreeSet;

/// Keys that name an input; a link has at most one kind of them.
const INPUT_KEYS: [&str; 4] = ["example", "tree", "session", "from"];

/// Shortest unknown key that gets a suggestion: shorter keys, such as Auspice's `c`, are within
/// edit distance 2 of too many keys.
const SUGGESTION_MIN_CHARS: usize = 4;

/// Largest edit distance between an unknown key and the key it suggests.
const SUGGESTION_MAX_DISTANCE: usize = 2;

/// Read the launch of the key-value pairs of a link, in the order of the link. `view_keys` are
/// the keys that the web app reads itself, which are neither errors nor ignored. Every problem is
/// an error at the key it concerns (`gamma`, `tree[1]`); a value of the right type outside the
/// bounds of a setting is no error here, so that the settings form shows it at its field.
pub fn parse_launch(pairs: &[(String, String)], view_keys: &[String]) -> LaunchParse {
  let mut reader = LaunchReader::default();
  let mut after_query_location = false;
  for (key, value) in pairs {
    let known = reader.read(key, value);
    if !known && !view_keys.contains(key) {
      reader.ignored.push(IgnoredKey {
        key: key.clone(),
        suggestion: suggestion(key, view_keys),
        after_location_query: after_query_location,
      });
    }
    after_query_location = matches!(key.as_str(), "tree" | "session") && value.contains('?');
  }
  reader.finish()
}

/// The state of [`parse_launch`] while it reads the pairs of a link.
#[derive(Default)]
struct LaunchReader {
  /// The kind of the first input key of the link.
  kind: Option<InputKind>,
  /// The input of a single-value input key, `None` when its value is invalid.
  input: Option<LaunchInput>,
  /// The `tree` values, in their order.
  trees: Vec<String>,
  patch: SettingsPatch,
  /// Keys of value settings given so far, so that a repeated key is an error.
  seen: BTreeSet<&'static str>,
  run: bool,
  /// The first key that needs an input: a setting or `run`.
  needs_input: Option<String>,
  /// The link names a format version that this build cannot read.
  too_new: bool,
  errors: Vec<ValidationError>,
  ignored: Vec<IgnoredKey>,
}

impl LaunchReader {
  /// Read one pair; `false` for a key that is no launch key.
  fn read(&mut self, key: &str, value: &str) -> bool {
    if let Some(kind) = input_kind(key) {
      self.read_input(kind, key, value);
    } else if let Some((setting, sets)) = setting_key(key) {
      self.read_setting(setting, sets, key, value);
      self.needs_input.get_or_insert_with(|| key.to_owned());
    } else if key == "run" {
      let on = self.flag(key, value);
      self.run |= on;
      self.needs_input.get_or_insert_with(|| key.to_owned());
    } else if key == "v" {
      self.read_version(value);
    } else {
      return false;
    }
    true
  }

  fn read_input(&mut self, kind: InputKind, key: &str, value: &str) {
    match self.kind {
      Some(first) if first != kind => {
        self.error(
          key,
          "use one input: an example, trees, a session file, or a message".to_owned(),
        );
        return;
      },
      Some(_) if kind != InputKind::Trees => {
        self.error(key, format!("{key} is given twice"));
        return;
      },
      _ => {},
    }
    self.kind = Some(kind);
    self.input = match kind {
      InputKind::Trees => {
        self.trees.push(value.to_owned());
        None
      },
      InputKind::Example => self.example(value).map(|id| LaunchInput::Example { id }),
      InputKind::Session => match parse_location(value) {
        Ok(location) => Some(LaunchInput::Session { location }),
        Err(e) => {
          self.error(key, e.to_string());
          None
        },
      },
      InputKind::Message => match value {
        "opener" => Some(LaunchInput::Message {
          source: MessageSource::Opener,
        }),
        "parent" => Some(LaunchInput::Message {
          source: MessageSource::Parent,
        }),
        _ => {
          self.error(key, format!("from must be opener or parent, got {value:?}"));
          None
        },
      },
    };
  }

  /// The example `id`, or an error with the closest id.
  fn example(&mut self, id: &str) -> Option<String> {
    if examples::example(id).is_some() {
      return Some(id.to_owned());
    }
    let hint = closest(id, examples::EXAMPLES.iter().map(|e| e.id))
      .map(|s| format!("; did you mean {s:?}?"))
      .unwrap_or_default();
    self.error("example", format!("there is no example {id:?}{hint}"));
    None
  }

  /// Read a setting key; `sets` is the value a flag key sets, `None` for a key with a value. A
  /// flag and its opposite may both occur, the last one wins; a key with a value occurs once.
  fn read_setting(&mut self, setting: &SettingKey, sets: Option<bool>, key: &str, value: &str) {
    if let Some(on) = sets {
      if self.flag(key, value) {
        let field = match setting.setting {
          SettingName::PreResolve => &mut self.patch.pre_resolve,
          SettingName::FinalRound => &mut self.patch.final_round,
          SettingName::Likelihood => &mut self.patch.likelihood,
          SettingName::Naive => &mut self.patch.naive,
          SettingName::Gamma
          | SettingName::SeqLengths
          | SettingName::NMcmcIt
          | SettingName::Resolve
          | SettingName::Rounds
          | SettingName::Seed => return,
        };
        *field = Some(on);
      }
      return;
    }
    if !self.seen.insert(setting.key) {
      self.error(key, format!("{key} is given twice"));
      return;
    }
    let p = &mut self.patch;
    let read = match setting.setting {
      SettingName::Gamma => parse_number(key, value).map(|v| p.gamma = Some(v)),
      SettingName::SeqLengths => parse_numbers(key, value).map(|v| p.seq_lengths = Some(v)),
      SettingName::NMcmcIt => parse_integer(key, value).map(|v| p.n_mcmc_it = Some(v)),
      SettingName::Rounds => parse_integer(key, value).map(|v| p.rounds = Some(v)),
      SettingName::Seed => parse_integer(key, value).map(|v| p.seed = Some(v)),
      SettingName::Resolve => parse_mode(value).map(|v| p.resolve = Some(v)),
      SettingName::PreResolve | SettingName::FinalRound | SettingName::Likelihood | SettingName::Naive => Ok(()),
    };
    if let Err(e) = read {
      self.error(key, e);
    }
  }

  /// Whether the flag `key` has no value, as a flag must; an error otherwise.
  fn flag(&mut self, key: &str, value: &str) -> bool {
    if !value.is_empty() {
      self.error(key, format!("{key} takes no value, got {value:?}"));
    }
    value.is_empty()
  }

  fn read_version(&mut self, value: &str) {
    match value.parse::<u64>() {
      Ok(v) if v <= FORMAT_VERSION => {},
      Ok(_) => {
        self.too_new = true;
        self.error("v", "This link needs a newer version of TreeKnit".to_owned());
      },
      #[cfg_attr(
        dylint_lib = "custom",
        expect(
          error_dropped_by_pattern,
          reason = "the message names the key and the value; the parse error of `str` adds nothing to it"
        )
      )]
      Err(_) => self.error("v", format!("v must be a whole number, got {value:?}")),
    }
  }

  fn error(&mut self, key: &str, message: String) {
    self
      .errors
      .push(ValidationError::at(Field::LinkKey { key: key.to_owned() }, message));
  }

  fn finish(mut self) -> LaunchParse {
    let input = match self.kind {
      Some(InputKind::Trees) => self.tree_input(),
      Some(_) => self.input.take(),
      None => {
        if let Some(key) = self.needs_input.take() {
          self.error(
            &key,
            format!("{key} needs an input: add example=, tree=, session=, or from= to the link"),
          );
        }
        None
      },
    };
    let launch = input.filter(|_| !self.too_new).map(|input| Launch {
      input,
      settings: self.patch,
      run: self.run,
    });
    LaunchParse {
      launch,
      errors: self.errors,
      ignored: self.ignored,
    }
  }

  /// The trees of the `tree` values, labeled, or `None` when a location is invalid or the link
  /// has fewer than two trees.
  fn tree_input(&mut self) -> Option<LaunchInput> {
    let mut parsed = Vec::new();
    let mut valid = true;
    for (i, value) in self.trees.iter().enumerate() {
      let token = split_label(value);
      match parse_location(token.rest) {
        Ok(location) => parsed.push((token.label.map(str::to_owned), location)),
        Err(e) => {
          self
            .errors
            .push(ValidationError::at(Field::LinkTree { index: i }, e.to_string()));
          valid = false;
        },
      }
    }
    if self.trees.len() < MIN_TREES {
      self.errors.push(ValidationError::at(
        Field::LinkKey { key: "tree".to_owned() },
        format!("a link needs at least two trees, got {}", self.trees.len()),
      ));
      valid = false;
    }
    if !valid {
      return None;
    }
    let labels = link_labels(&parsed);
    let trees = parsed
      .into_iter()
      .zip(labels)
      .map(|((_, location), label)| LaunchTree { label, location })
      .collect();
    Some(LaunchInput::Trees { trees })
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputKind {
  Example,
  Trees,
  Session,
  Message,
}

/// The input kind of a key, `None` for a key that names no input.
fn input_kind(key: &str) -> Option<InputKind> {
  match key {
    "example" => Some(InputKind::Example),
    "tree" => Some(InputKind::Trees),
    "session" => Some(InputKind::Session),
    "from" => Some(InputKind::Message),
    _ => None,
  }
}

/// The setting of a key and, for a flag, the value it sets.
fn setting_key(key: &str) -> Option<(&'static SettingKey, Option<bool>)> {
  SETTING_KEYS.iter().find_map(|s| match s.value {
    KeyValue::Flag { key_sets, .. } if s.key == key => Some((s, Some(key_sets))),
    KeyValue::Flag { key_sets, opposite } if opposite == key => Some((s, Some(!key_sets))),
    KeyValue::Flag { .. } => None,
    _ if s.key == key => Some((s, None)),
    _ => None,
  })
}

/// Labels of the trees of a link: the given labels, and for the others `analysis::tree_labels`
/// of the file names of their locations, with the given labels as existing labels.
fn link_labels(trees: &[(Option<String>, LinkLocation)]) -> Vec<String> {
  let given: Vec<String> = trees.iter().filter_map(|(label, _)| label.clone()).collect();
  let names: Vec<String> = trees
    .iter()
    .filter(|(label, _)| label.is_none())
    .map(|(_, location)| location_file_name(location))
    .collect();
  let mut automatic = analysis::tree_labels(&names, &given).into_iter();
  trees
    .iter()
    .map(|(label, _)| label.clone().or_else(|| automatic.next()).unwrap_or_default())
    .collect()
}

fn location_file_name(location: &LinkLocation) -> String {
  match location {
    LinkLocation::Url { file_name, .. } => file_name.clone(),
    LinkLocation::Data { .. } => String::new(),
  }
}

#[cfg_attr(
  dylint_lib = "custom",
  expect(
    result_defaulted,
    reason = "the message names the key and the value; the parse error of `str` adds nothing to it"
  )
)]
fn parse_number(key: &str, value: &str) -> Result<f64, String> {
  value
    .trim()
    .parse::<f64>()
    .ok()
    .filter(|v| v.is_finite())
    .ok_or_else(|| format!("{key} must be a number, got {value:?}"))
}

#[cfg_attr(
  dylint_lib = "custom",
  expect(
    result_defaulted,
    reason = "the message names the key and the value; the parse error of `str` adds nothing to it"
  )
)]
fn parse_numbers(key: &str, value: &str) -> Result<Vec<f64>, String> {
  value
    .split(',')
    .map(|v| v.trim().parse::<f64>().ok().filter(|v| v.is_finite()))
    .collect::<Option<Vec<_>>>()
    .ok_or_else(|| format!("{key} must be numbers separated by commas, such as 1701,1410, got {value:?}"))
}

/// A whole number of a link, at most `analysis::MAX_SEED`, the largest integer that a JavaScript
/// number holds exactly, so the web app receives the value of the link.
fn parse_integer(key: &str, value: &str) -> Result<u64, String> {
  match value.trim().parse::<u64>() {
    Ok(v) if v <= analysis::MAX_SEED => Ok(v),
    Ok(_) => Err(format!("{key} must be at most {}, got {value}", analysis::MAX_SEED)),
    #[cfg_attr(
      dylint_lib = "custom",
      expect(
        error_dropped_by_pattern,
        reason = "the message names the key and the value; the parse error of `str` adds nothing to it"
      )
    )]
    Err(_) => Err(format!("{key} must be a whole number, got {value:?}")),
  }
}

fn parse_mode(value: &str) -> Result<ResolveMode, String> {
  from_wire_name(value).ok_or_else(|| format!("resolve must be matched, strict, liberal, or none, got {value:?}"))
}

/// The key of the launch keys and `view_keys` that `key` most likely misspells.
fn suggestion(key: &str, view_keys: &[String]) -> Option<String> {
  if key.chars().count() < SUGGESTION_MIN_CHARS {
    return None;
  }
  let launch = INPUT_KEYS
    .into_iter()
    .chain(SETTING_KEYS.iter().flat_map(|s| [Some(s.key), s.opposite()]).flatten())
    .chain(["run"]);
  closest(key, launch.chain(view_keys.iter().map(String::as_str))).map(str::to_owned)
}

/// The candidate within [`SUGGESTION_MAX_DISTANCE`] edits of `word` with the fewest edits, the
/// first one on a tie.
fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
  candidates
    .map(|c| (strsim::levenshtein(word, c), c))
    .filter(|(d, _)| *d <= SUGGESTION_MAX_DISTANCE)
    .min_by_key(|(d, _)| *d)
    .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::launch::location::fetch_url;
  use crate::launch::{UrlPlace, query_pairs, url_place};
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  fn pairs(query: &str) -> Vec<(String, String)> {
    query_pairs(query)
  }

  fn parse(query: &str) -> LaunchParse {
    parse_launch(
      &pairs(query),
      &["view".to_owned(), "auspice".to_owned(), "leaf".to_owned()],
    )
  }

  fn url(u: &str) -> LinkLocation {
    let UrlPlace { host, file_name } = url_place(u);
    LinkLocation::Url {
      url: u.to_owned(),
      fetch: fetch_url(u),
      host,
      file_name,
    }
  }

  fn tree(label: &str, location: LinkLocation) -> LaunchTree {
    LaunchTree {
      label: label.to_owned(),
      location,
    }
  }

  fn error(key: &str, message: &str) -> ValidationError {
    ValidationError::at(Field::LinkKey { key: key.to_owned() }, message)
  }

  fn launch(input: LaunchInput, settings: SettingsPatch, run: bool) -> Option<Launch> {
    Some(Launch { input, settings, run })
  }

  #[test]
  fn example_with_settings_and_run() {
    let expected = LaunchParse {
      launch: launch(
        LaunchInput::Example {
          id: "h3n2-2017".to_owned(),
        },
        SettingsPatch {
          gamma: Some(3.0),
          resolve: Some(ResolveMode::Strict),
          final_round: Some(false),
          ..SettingsPatch::default()
        },
        true,
      ),
      errors: vec![],
      ignored: vec![],
    };
    assert_eq!(
      expected,
      parse("example=h3n2-2017&gamma=3&resolve=strict&no-final-round&run&view=mccs")
    );
  }

  #[test]
  fn trees_get_labels_from_their_file_names_next_to_given_labels() {
    let parsed =
      parse("tree=https://x/a/ha.nwk&tree=ha=https://x/b/ha.nwk&tree=https://x/c/&tree=data:,(A,B);&tree=data:,(A,C);");
    let expected = LaunchInput::Trees {
      trees: vec![
        tree("ha_2", url("https://x/a/ha.nwk")),
        tree("ha", url("https://x/b/ha.nwk")),
        tree("c", url("https://x/c/")),
        tree(
          "tree",
          LinkLocation::Data {
            text: "(A,B);".to_owned(),
          },
        ),
        tree(
          "tree_2",
          LinkLocation::Data {
            text: "(A,C);".to_owned(),
          },
        ),
      ],
    };
    assert_eq!(
      (Some(expected), vec![]),
      (parsed.launch.map(|l| l.input), parsed.errors)
    );
  }

  #[test]
  fn url_without_a_file_name_gives_the_label_tree() {
    let parsed = parse("tree=https://example.org&tree=https://example.org/");
    let labels: Vec<String> = match parsed.launch.unwrap().input {
      LaunchInput::Trees { trees } => trees.into_iter().map(|t| t.label).collect(),
      #[cfg_attr(
        dylint_lib = "mordant",
        expect(wildcard_over_own_enum, reason = "the test fails on every other variant")
      )]
      other => panic!("expected trees, got {other:?}"),
    };
    assert_eq!(vec!["tree", "tree_2"], labels);
  }

  #[test]
  fn invalid_tree_location_is_an_error_at_its_tree_and_stops_the_launch() {
    let parsed = parse("tree=https://x/ha.nwk&tree=ha.nwk");
    let expected = vec![ValidationError::at(
      Field::LinkTree { index: 1 },
      "\"ha.nwk\" is neither an https: address nor a data: text",
    )];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[test]
  fn one_tree_is_too_few() {
    let parsed = parse("tree=https://x/ha.nwk&run");
    let expected = vec![error("tree", "a link needs at least two trees, got 1")];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::trees_then_example("tree=https://x/a&tree=https://x/b&example=h3n2-2017", "example")]
  #[case::example_then_tree( "example=h3n2-2017&tree=https://x/a",                "tree")]
  #[case::example_then_from( "example=h3n2-2017&from=opener",                     "from")]
  #[trace]
  fn a_second_input_kind_is_an_error_and_the_first_one_loads(#[case] query: &str, #[case] key: &str) {
    let parsed = parse(query);
    let expected = vec![error(key, "use one input: an example, trees, a session file, or a message")];
    assert_eq!((true, expected), (parsed.launch.is_some(), parsed.errors));
  }

  #[test]
  fn a_repeated_example_is_an_error() {
    let parsed = parse("example=h3n2-2017&example=5-leaves");
    let expected = launch(
      LaunchInput::Example {
        id: "h3n2-2017".to_owned(),
      },
      SettingsPatch::default(),
      false,
    );
    assert_eq!(
      (expected, vec![error("example", "example is given twice")]),
      (parsed.launch, parsed.errors)
    );
  }

  #[test]
  fn unknown_example_suggests_the_closest_id() {
    let parsed = parse("example=h3n2-2071");
    let expected = vec![error(
      "example",
      "there is no example \"h3n2-2071\"; did you mean \"h3n2-2017\"?",
    )];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::absent( "example=5-leaves",      true,  vec![])]
  #[case::one(    "example=5-leaves&v=1",  true,  vec![])]
  #[case::newer(  "example=5-leaves&v=2",  false, vec![error("v", "This link needs a newer version of TreeKnit")])]
  #[case::invalid("example=5-leaves&v=x",  true,  vec![error("v", "v must be a whole number, got \"x\"")])]
  #[trace]
  fn version_one_and_absent_versions_load(#[case] query: &str, #[case] loads: bool, #[case] errors: Vec<ValidationError>) {
    let parsed = parse(query);
    assert_eq!((loads, errors), (parsed.launch.is_some(), parsed.errors));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::gamma_text(    "gamma=abc",       error("gamma", "gamma must be a number, got \"abc\""))]
  #[case::gamma_infinite("gamma=inf",       error("gamma", "gamma must be a number, got \"inf\""))]
  #[case::resolve(       "resolve=fast",    error("resolve", "resolve must be matched, strict, liberal, or none, got \"fast\""))]
  #[case::rounds(        "rounds=1.5",      error("rounds", "rounds must be a whole number, got \"1.5\""))]
  #[case::seed_large(    "seed=9007199254740992", error("seed", "seed must be at most 9007199254740991, got 9007199254740992"))]
  #[case::lengths(       "seq-lengths=1701 1410", error("seq-lengths", "seq-lengths must be numbers separated by commas, such as 1701,1410, got \"1701 1410\""))]
  #[case::flag_value(    "naive=1",         error("naive", "naive takes no value, got \"1\""))]
  #[case::run_value(     "run=true",        error("run", "run takes no value, got \"true\""))]
  #[trace]
  fn values_of_the_wrong_type_are_errors_and_leave_the_setting_unset(#[case] query: &str, #[case] expected: ValidationError) {
    let parsed = parse(&format!("example=5-leaves&{query}"));
    let launch = parsed.launch.map(|l| (l.settings, l.run));
    assert_eq!((vec![expected], Some((SettingsPatch::default(), false))), (parsed.errors, launch));
  }

  #[test]
  fn a_repeated_setting_is_an_error_and_keeps_the_first_value() {
    let parsed = parse("example=5-leaves&gamma=1&gamma=2");
    let expected = SettingsPatch {
      gamma: Some(1.0),
      ..SettingsPatch::default()
    };
    assert_eq!(
      (vec![error("gamma", "gamma is given twice")], Some(expected)),
      (parsed.errors, parsed.launch.map(|l| l.settings))
    );
  }

  #[test]
  fn values_outside_their_bounds_go_into_the_patch() {
    let parsed = parse("example=5-leaves&gamma=-1&rounds=0&seq-lengths=1701,0,-3");
    let expected = SettingsPatch {
      gamma: Some(-1.0),
      rounds: Some(0),
      seq_lengths: Some(vec![1701.0, 0.0, -3.0]),
      ..SettingsPatch::default()
    };
    assert_eq!(
      (vec![], Some(expected)),
      (parsed.errors, parsed.launch.map(|l| l.settings))
    );
  }

  #[test]
  fn the_last_of_a_flag_and_its_opposite_wins() {
    let parsed = parse("example=5-leaves&pre-resolve&no-pre-resolve&no-naive&naive&likelihood&final-round");
    let expected = SettingsPatch {
      pre_resolve: Some(false),
      naive: Some(true),
      likelihood: Some(true),
      final_round: Some(true),
      ..SettingsPatch::default()
    };
    assert_eq!(
      (vec![], Some(expected)),
      (parsed.errors, parsed.launch.map(|l| l.settings))
    );
  }

  #[test]
  fn settings_without_an_input_are_an_error_at_the_first_one() {
    let parsed = parse("view=mccs&gamma=3&run");
    let expected = vec![error(
      "gamma",
      "gamma needs an input: add example=, tree=, session=, or from= to the link",
    )];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[test]
  fn a_link_with_display_keys_only_has_no_launch() {
    let expected = LaunchParse {
      launch: None,
      errors: vec![],
      ignored: vec![],
    };
    assert_eq!(expected, parse("view=mccs&leaf=A&v=1"));
  }

  #[test]
  fn unknown_keys_are_ignored_with_a_suggestion() {
    let parsed = parse("example=5-leaves&gama=3&aupsice=c%3Dmcc&c=mcc&utm_source=paper");
    let ignored = |key: &str, suggestion: Option<&str>| IgnoredKey {
      key: key.to_owned(),
      suggestion: suggestion.map(str::to_owned),
      after_location_query: false,
    };
    let expected = vec![
      ignored("gama", Some("gamma")),
      ignored("aupsice", Some("auspice")),
      ignored("c", None),
      ignored("utm_source", None),
    ];
    assert_eq!(expected, parsed.ignored);
  }

  #[test]
  fn a_key_after_a_tree_location_with_a_query_carries_the_hint() {
    let parsed = parse("tree=https://x/ha.nwk?token=1&sig=2&tree=https://x/na.nwk&other=1");
    let expected = vec![
      IgnoredKey {
        key: "sig".to_owned(),
        suggestion: None,
        after_location_query: true,
      },
      IgnoredKey {
        key: "other".to_owned(),
        suggestion: None,
        after_location_query: false,
      },
    ];
    assert_eq!(expected, parsed.ignored);
  }

  #[test]
  fn session_and_message_inputs() {
    let session = parse("session=https://zenodo.org/records/1/files/s.json&seed=7");
    let expected = launch(
      LaunchInput::Session {
        location: url("https://zenodo.org/records/1/files/s.json"),
      },
      SettingsPatch {
        seed: Some(7),
        ..SettingsPatch::default()
      },
      false,
    );
    assert_eq!(expected, session.launch);
    let message = parse("from=parent&run");
    let expected = launch(
      LaunchInput::Message {
        source: MessageSource::Parent,
      },
      SettingsPatch::default(),
      true,
    );
    assert_eq!(expected, message.launch);
    assert_eq!(
      vec![error("from", "from must be opener or parent, got \"top\"")],
      parse("from=top").errors
    );
  }
}
