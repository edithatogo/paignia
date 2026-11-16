use std::error::Error;

use paignia_core::ValidationError;
use paignia_solve::SolveError;

#[test]
fn solver_errors_preserve_display_and_typed_sources() {
    let cause = ValidationError::EmptyIdentifier {
        path: "game.id".to_owned(),
    };
    let error = SolveError::InvalidGame(cause.clone());

    assert_eq!(
        error.to_string(),
        "invalid game: empty identifier at game.id"
    );
    assert_eq!(
        error
            .source()
            .and_then(|source| source.downcast_ref::<ValidationError>()),
        Some(&cause)
    );

    let tolerance_error = SolveError::InvalidTolerance(0.25);
    assert_eq!(tolerance_error.to_string(), "invalid solver tolerance 0.25");
    assert!(tolerance_error.source().is_none());
}
