use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;

use paignia_core::{
    ActionEdge, ActionId, Game, GameSpecification, Node, NodeId, PlayerId, STATIC_GAME_PROFILE_V1,
};
use paignia_solve::backward_induction;
use serde::Deserialize;
use serde_json::json;

// The receiving replay independently checks both locked dependencies against this revision.
const PAIGNIA_REVISION: &str = "a04a19da06cc50e310a172574ed66f62000b891a";
const MAX_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Specification {
    schema_version: String,
    id: String,
    game: Graph,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Graph {
    root: String,
    nodes: BTreeMap<String, InputNode>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum InputNode {
    Decision { player: String, edges: Vec<Edge> },
    Terminal { payoffs: BTreeMap<String, f64> },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edge {
    action: String,
    target: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().collect();
    let [_, command, request_flag, request, output_flag, output] = args.as_slice() else {
        return Err("expected solve --request FILE --output FILE".into());
    };
    if command != "solve" || request_flag != "--request" || output_flag != "--output" {
        return Err("expected solve --request FILE --output FILE".into());
    }

    let mut input = Vec::new();
    File::open(request)?
        .take(u64::try_from(MAX_REQUEST_BYTES + 1)?)
        .read_to_end(&mut input)?;
    if input.len() > MAX_REQUEST_BYTES {
        return Err("Dapane request exceeds 64 KiB".into());
    }
    let Specification {
        schema_version,
        id,
        game: Graph { root, nodes },
    } = serde_json::from_slice(&input)?;
    let Some(InputNode::Decision { player, edges }) = nodes.get(&root) else {
        return Err("Dapane requires a decision root".into());
    };
    if !(2..=32).contains(&edges.len()) || nodes.len() != edges.len() + 1 {
        return Err("Dapane requires two to 32 actions with one terminal per action".into());
    }
    for (node, entry) in &nodes {
        if node != &root
            && !matches!(entry, InputNode::Terminal { payoffs }
                if payoffs.len() == 1 && payoffs.contains_key(player))
        {
            return Err(
                "Dapane requires one explicit utility for the controller at each terminal".into(),
            );
        }
    }

    let player = PlayerId(player.clone());
    let root = NodeId(root);
    let specification = GameSpecification {
        schema_version,
        id,
        game: Game {
            root: root.clone(),
            nodes: nodes
                .into_iter()
                .map(|(id, node)| {
                    let native = match node {
                        InputNode::Decision { player, edges } => Node::Decision {
                            player: PlayerId(player),
                            edges: edges
                                .into_iter()
                                .map(|edge| ActionEdge {
                                    action: ActionId(edge.action),
                                    target: NodeId(edge.target),
                                })
                                .collect(),
                        },
                        InputNode::Terminal { payoffs } => Node::Terminal {
                            payoffs: payoffs
                                .into_iter()
                                .map(|(player, payoff)| (PlayerId(player), payoff))
                                .collect(),
                        },
                    };
                    (NodeId(id), native)
                })
                .collect(),
        },
    };
    specification.validate_for_profile(STATIC_GAME_PROFILE_V1)?;
    let solution = backward_induction(&specification.game)?;
    let action = solution.choices.get(&root).ok_or("missing root choice")?;
    let payoff = solution
        .expected_payoffs
        .get(&player)
        .ok_or("missing controller utility")?;
    let envelope = json!({
        "solution": {
            "profile": STATIC_GAME_PROFILE_V1,
            "source_revision": PAIGNIA_REVISION,
            "root": root.0,
            "selected_action": action.0,
            "expected_payoff": payoff,
        },
    });
    fs::write(output, serde_json::to_vec(&envelope)?)?;
    Ok(())
}
