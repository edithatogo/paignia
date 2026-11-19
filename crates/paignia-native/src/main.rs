//! Python-free native CLI for Paignia game validation and solution.
//!
//! Usage: paignia-native solve --request FILE --output FILE
//!
//! Reads a JSON GameSpecification from --request, validates it, runs
//! backward induction, and writes the JSON solution to --output.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn parse_args() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 5 || !matches!(args[0].as_str(), "solve") {
        return Err("usage: paignia-native solve --request FILE --output FILE".into());
    }
    let mut request_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--request" if i + 1 < args.len() => {
                request_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--output" if i + 1 < args.len() => {
                output_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            other => return Err(format!("unknown or incomplete argument: {other}")),
        }
    }
    Ok((
        request_path.ok_or_else(|| "missing --request FILE".to_string())?,
        output_path.ok_or_else(|| "missing --output FILE".to_string())?,
    ))
}

fn main() -> ExitCode {
    let (request_path, output_path) = match parse_args() {
        Ok(parsed) => parsed,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };

    let raw = match fs::read_to_string(&request_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("failed to read {}: {err}", request_path.display());
            return ExitCode::from(1);
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("malformed JSON request: {err}");
            return ExitCode::from(1);
        }
    };

    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            eprintln!("request JSON must be an object");
            return ExitCode::from(1);
        }
    };

    let schema_version = obj
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if schema_version != "1.0.0" {
        eprintln!("unsupported schema version: {schema_version}");
        return ExitCode::from(1);
    }

    let game_id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let game_obj = match obj.get("game").and_then(|v| v.as_object()) {
        Some(o) => o,
        None => {
            eprintln!("game object is required");
            return ExitCode::from(1);
        }
    };

    let root_id = match game_obj.get("root").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => {
            eprintln!("game.root is required");
            return ExitCode::from(1);
        }
    };

    let nodes_obj = match game_obj.get("nodes").and_then(|v| v.as_object()) {
        Some(o) => o,
        None => {
            eprintln!("game.nodes is required");
            return ExitCode::from(1);
        }
    };

    // Build NodeId -> Node mapping manually
    let mut nodes: BTreeMap<paignia_core::NodeId, paignia_core::Node> = BTreeMap::new();
    for (node_id_str, node_value) in nodes_obj {
        let node_id = paignia_core::NodeId(node_id_str.clone());
        let node = match node_value.get("kind").and_then(|v| v.as_str()) {
            Some("Decision") => {
                let player = paignia_core::PlayerId(
                    node_value
                        .get("player")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                );
                let edges_arr = node_value
                    .get("edges")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let mut edges = Vec::new();
                for edge in edges_arr {
                    let edge_obj = match edge.as_object() {
                        Some(o) => o,
                        None => continue,
                    };
                    let action = paignia_core::ActionId(
                        edge_obj
                            .get("action")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    );
                    let target = paignia_core::NodeId(
                        edge_obj
                            .get("target")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    );
                    edges.push(paignia_core::ActionEdge { action, target });
                }
                paignia_core::Node::Decision { player, edges }
            }
            Some("Chance") => {
                let edges_arr = node_value
                    .get("edges")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let mut edges = Vec::new();
                for edge in edges_arr {
                    let edge_obj = match edge.as_object() {
                        Some(o) => o,
                        None => continue,
                    };
                    let probability = edge_obj
                        .get("probability")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let target = paignia_core::NodeId(
                        edge_obj
                            .get("target")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    );
                    edges.push(paignia_core::ChanceEdge {
                        probability,
                        target,
                    });
                }
                paignia_core::Node::Chance { edges }
            }
            Some("Terminal") => {
                let empty_map = serde_json::Map::new();
                let payoffs_obj = match node_value.get("payoffs").and_then(|v| v.as_object()) {
                    Some(o) => o,
                    None => &empty_map,
                };
                let mut payoffs = BTreeMap::new();
                for (player_id_str, payoff_value) in payoffs_obj {
                    let player_id = paignia_core::PlayerId(player_id_str.clone());
                    let payoff = payoff_value.as_f64().unwrap_or(0.0);
                    payoffs.insert(player_id, payoff);
                }
                paignia_core::Node::Terminal { payoffs }
            }
            _ => {
                eprintln!("unknown node kind in node {}", node_id_str);
                return ExitCode::from(1);
            }
        };
        nodes.insert(node_id, node);
    }

    let spec = paignia_core::GameSpecification {
        schema_version: schema_version.to_string(),
        id: game_id,
        game: paignia_core::Game {
            root: paignia_core::NodeId(root_id),
            nodes,
        },
    };

    if let Err(err) = spec.validate() {
        eprintln!("game specification validation failed: {err}");
        return ExitCode::from(1);
    }

    let solution = match paignia_solve::backward_induction(&spec.game) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("game solving failed: {err}");
            return ExitCode::from(1);
        }
    };

    // Build output JSON manually
    let mut expected_payoffs = serde_json::Map::new();
    for (player, payoff) in &solution.expected_payoffs {
        expected_payoffs.insert(player.0.clone(), serde_json::json!(payoff));
    }

    let mut choices = serde_json::Map::new();
    for (node, action) in &solution.choices {
        choices.insert(node.0.clone(), serde_json::json!(action.0));
    }

    let mut trace = Vec::new();
    for step in &solution.trace {
        let mut trace_step = serde_json::Map::new();
        trace_step.insert("sequence".to_string(), serde_json::json!(step.sequence));
        trace_step.insert("node".to_string(), serde_json::json!(step.node.0));
        trace_step.insert(
            "kind".to_string(),
            serde_json::json!(format!("{:?}", step.kind)),
        );
        trace_step.insert(
            "selected_action".to_string(),
            step.selected_action
                .as_ref()
                .map(|a| serde_json::json!(a.0))
                .unwrap_or(serde_json::Value::Null),
        );
        let mut step_payoffs = serde_json::Map::new();
        for (player, payoff) in &step.expected_payoffs {
            step_payoffs.insert(player.0.clone(), serde_json::json!(payoff));
        }
        trace_step.insert(
            "expected_payoffs".to_string(),
            serde_json::Value::Object(step_payoffs),
        );
        trace.push(serde_json::Value::Object(trace_step));
    }

    let output = serde_json::json!({
        "schema_version": "1.0.0",
        "game_id": spec.id,
        "solution": {
            "expected_payoffs": serde_json::Value::Object(expected_payoffs),
            "choices": serde_json::Value::Object(choices),
            "tolerance": solution.tolerance,
            "tie_policy": format!("{:?}", solution.tie_policy),
            "diagnostics": {
                "visited_nodes": solution.diagnostics.visited_nodes,
                "resolved_ties": solution.diagnostics.resolved_ties,
            },
            "trace": serde_json::Value::Array(trace),
        },
        "runtime": "rust_native",
    });

    if let Err(err) = fs::write(&output_path, &serde_json::to_string(&output).unwrap()) {
        eprintln!("failed to write {}: {err}", output_path.display());
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}
