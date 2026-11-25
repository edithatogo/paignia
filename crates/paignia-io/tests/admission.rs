use paignia_io::{
    solve_profile_json, solve_static_json, Error, ProfileError, DISCOUNTED_PROFILE,
    MAX_REQUEST_BYTES,
};
use serde_json::{json, Value};

#[test]
fn both_wire_entry_points_check_size_before_decoding() {
    let oversized = vec![b' '; MAX_REQUEST_BYTES + 1];
    assert!(matches!(
        solve_static_json(&oversized),
        Err(Error::RequestTooLarge)
    ));
    assert!(matches!(
        solve_profile_json(&oversized),
        Err(ProfileError::RequestTooLarge)
    ));

    let at_limit = &oversized[..MAX_REQUEST_BYTES];
    assert!(matches!(solve_static_json(at_limit), Err(Error::Json(_))));
    assert!(matches!(
        solve_profile_json(at_limit),
        Err(ProfileError::Json(_))
    ));
}

fn discounted_request() -> Value {
    let outcome = json!({ "reward": 2.0, "transition": [1.0] });
    json!({
        "version": "1.0.0",
        "semantic_profile": DISCOUNTED_PROFILE,
        "source": {
            "ontology_revision": "a".repeat(40),
            "rdf_sha256": "b".repeat(64),
            "representation": "urn:game:representation"
        },
        "specification": "urn:game:specification",
        "instance": "urn:game:instance",
        "run": "urn:game:run",
        "game": {
            "kind": "discounted",
            "players": [
                { "id": "urn:row", "actions": ["urn:row:left", "urn:row:right"] },
                { "id": "urn:col", "actions": ["urn:col:up", "urn:col:down"] }
            ],
            "states": [{
                "id": "urn:state:absorbing",
                "outcomes": [[outcome.clone(), outcome.clone()], [outcome.clone(), outcome]]
            }],
            "initial": [1.0],
            "discount": 0.5,
            "method": { "kind": "evaluate", "policy": [[[1.0, 0.0], [0.0, 1.0]]] },
            "max_iterations": 100,
            "value_tolerance": 1e-10
        }
    })
}

#[test]
fn discounted_wire_retains_identity_and_distinguishes_result_status() {
    let mut request = discounted_request();
    let solve = |request: &Value| -> Value {
        serde_json::from_slice(&solve_profile_json(&serde_json::to_vec(request).unwrap()).unwrap())
            .unwrap()
    };
    let result = solve(&request);
    for field in [
        "source",
        "specification",
        "instance",
        "run",
        "semantic_profile",
    ] {
        assert_eq!(result[field], request[field]);
    }
    assert_eq!(result["method"], "stationary_policy_evaluation");
    assert_eq!(result["status"], "value_tolerance_reached");
    assert_eq!(
        result["result"]["policy"],
        request["game"]["method"]["policy"]
    );
    let values = &result["result"]["policy_values"];
    assert_eq!(values["state_values"][0]["state"], "urn:state:absorbing");
    assert!(
        (values["initial_value"].as_f64().unwrap() - 4.0).abs()
            <= values["initial_error_bound"].as_f64().unwrap()
    );

    request["game"]["method"] = json!({ "kind": "minimax" });
    request["game"]["max_iterations"] = json!(1);
    let result = solve(&request);
    assert_eq!(result["method"], "discounted_zero_sum_minimax");
    assert_eq!(result["status"], "iteration_limit");
    assert_eq!(
        result["result"]["deviation_bounds"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn discounted_wire_rejects_extra_fields_and_incomplete_transition_mass() {
    let mut request = discounted_request();
    request["game"]["method"]["reward_after_transition"] = json!(true);
    assert!(matches!(
        solve_profile_json(&serde_json::to_vec(&request).unwrap()),
        Err(ProfileError::Json(_))
    ));

    request["game"]["method"] = json!({ "kind": "minimax", "policy": [[[1.0, 0.0], [0.0, 1.0]]] });
    assert!(matches!(
        solve_profile_json(&serde_json::to_vec(&request).unwrap()),
        Err(ProfileError::Json(_))
    ));

    let mut request = discounted_request();
    request["game"]["states"][0]["outcomes"][0][0]["transition"] = json!([0.75]);
    assert!(matches!(
        solve_profile_json(&serde_json::to_vec(&request).unwrap()),
        Err(ProfileError::Discounted(_))
    ));
}
