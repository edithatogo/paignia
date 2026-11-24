use paignia_io::{solve_profile_json, solve_static_json, Error, ProfileError, MAX_REQUEST_BYTES};

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
