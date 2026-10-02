//! MCCs as JSON (the `MCC_dict` format of TreeKnit.jl), plus the legacy line format.

use serde_json::{json, Map, Value};
use treeknit_core::{PairResult, Taxa, Tree};

/// `{"MCC_dict": {"1": {"trees": [a, b], "mccs": [[...]], "imputed": [...]}, ...}}`.
///
/// `imputed` lists leaves present in only one tree of the pair, the tree they come from,
/// the (0-based) index of the MCC they were attached to, and whether the attachment was
/// ambiguous. It is omitted when empty.
pub fn to_json(pairs: &[PairResult], trees: &[Tree], taxa: &Taxa) -> Value {
    let mut dict = Map::new();
    for (k, p) in pairs.iter().enumerate() {
        let mut e = Map::new();
        e.insert("trees".into(), json!([trees[p.i].label, trees[p.j].label]));
        e.insert(
            "mccs".into(),
            json!(p.mccs.iter().map(|m| taxa.names_of(m)).collect::<Vec<_>>()),
        );
        let mut imputed: Vec<Value> = Vec::new();
        for a in &p.attached {
            for &x in &a.leaves {
                imputed.push(json!({
                    "leaf": taxa.names[x],
                    "tree": trees[a.source].label,
                    "mcc": a.mcc,
                    "ambiguous": a.ambiguous,
                }));
            }
        }
        if !imputed.is_empty() {
            e.insert("imputed".into(), Value::Array(imputed));
        }
        dict.insert((k + 1).to_string(), Value::Object(e));
    }
    json!({ "MCC_dict": dict })
}

/// MCCs of one tree pair as leaf names, with the two tree labels.
pub type NamedPair = ([String; 2], Vec<Vec<String>>);

/// Read MCCs of all pairs from the JSON format.
pub fn from_json(v: &Value) -> Result<Vec<NamedPair>, String> {
    let dict = v.get("MCC_dict").and_then(Value::as_object).ok_or("missing MCC_dict")?;
    let mut out = Vec::new();
    for e in dict.values() {
        let trees: Vec<String> = serde_json::from_value(e["trees"].clone()).map_err(|e| e.to_string())?;
        let mccs: Vec<Vec<String>> = serde_json::from_value(e["mccs"].clone()).map_err(|e| e.to_string())?;
        let [a, b]: [String; 2] = trees.try_into().map_err(|_| "expected two tree labels")?;
        out.push(([a, b], mccs));
    }
    Ok(out)
}

/// Legacy format: one MCC per line, leaves separated by commas.
pub fn to_lines(mccs: &[Vec<String>]) -> String {
    mccs.iter().map(|m| m.join(",")).collect::<Vec<_>>().join("\n")
}

pub fn from_lines(s: &str) -> Vec<Vec<String>> {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|x| x.trim().to_string()).collect())
        .collect()
}
