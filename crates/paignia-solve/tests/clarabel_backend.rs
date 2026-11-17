//! Direct Clarabel qualification probes, not a Paignia CE implementation.
//!
//! Cases and exact objective bounds come from the unchanged rational fixture
//! `verification/numerical_oracles/correlated_equilibrium_v1.json`.
//! The solver assembly and recomputed diagnostics execute only in Rust.
#![cfg(feature = "correlated-equilibrium")]
// Exact dyadic controls intentionally compare representable values directly.
#![allow(clippy::float_cmp)]

use clarabel::algebra::CscMatrix;
use clarabel::solver::{
    DefaultSettingsBuilder, DefaultSolution, DefaultSolver, IPSolver, NonnegativeConeT, ZeroConeT,
};

fn solve_ce(
    a: &[Vec<f64>],
    b: &[Vec<f64>],
    objective: &[f64],
    max_iter: u32,
) -> DefaultSolution<f64> {
    let rows = a.len();
    let cols = a[0].len();
    let size = rows * cols;
    assert_eq!(b.len(), rows);
    assert_eq!(objective.len(), size);
    assert!(a.iter().chain(b).all(|row| row.len() == cols));
    assert!(a
        .iter()
        .chain(b)
        .flatten()
        .chain(objective)
        .all(|x| x.is_finite()));
    let mut constraints = vec![vec![1.0; size]];
    for index in 0..size {
        let mut row = vec![0.0; size];
        row[index] = -1.0;
        constraints.push(row);
    }
    for recommendation in 0..rows {
        for deviation in 0..rows {
            if recommendation == deviation {
                continue;
            }
            let mut row = vec![0.0; size];
            for column in 0..cols {
                row[recommendation * cols + column] =
                    a[deviation][column] - a[recommendation][column];
            }
            constraints.push(row);
        }
    }
    for recommendation in 0..cols {
        for deviation in 0..cols {
            if recommendation == deviation {
                continue;
            }
            let mut row = vec![0.0; size];
            for player_row in 0..rows {
                row[player_row * cols + recommendation] =
                    b[player_row][deviation] - b[player_row][recommendation];
            }
            constraints.push(row);
        }
    }
    let count = constraints.len();
    let mut column_pointer = vec![0];
    let mut row_indices = vec![];
    let mut values = vec![];
    for column in 0..size {
        for (row, entries) in constraints.iter().enumerate() {
            if entries[column] != 0.0 {
                row_indices.push(row);
                values.push(entries[column]);
            }
        }
        column_pointer.push(values.len());
    }
    let matrix = CscMatrix::new(count, size, column_pointer, row_indices, values);
    assert!(matrix.check_format().is_ok());
    let p = CscMatrix::<f64>::zeros((size, size));
    let q: Vec<f64> = objective.iter().map(|x| -x).collect();
    let mut rhs = vec![0.0; count];
    rhs[0] = 1.0;
    let cones = [ZeroConeT(1), NonnegativeConeT(count - 1)];
    let settings = DefaultSettingsBuilder::default()
        .verbose(false)
        .max_iter(max_iter)
        .tol_feas(1e-10)
        .tol_gap_abs(1e-10)
        .tol_gap_rel(1e-10)
        .direct_solve_method("qdldl".to_string())
        .build()
        .unwrap();
    let mut solver = DefaultSolver::new(&p, &q, &matrix, &rhs, &cones, settings).unwrap();
    solver.solve();
    solver.solution
}

#[derive(Debug)]
struct Metrics {
    mass_error: f64,
    negative_mass: f64,
    weighted_gain: f64,
    conditional_gain: f64,
    objective: f64,
}

fn metrics(a: &[Vec<f64>], b: &[Vec<f64>], objective: &[f64], joint: &[f64]) -> Metrics {
    let rows = a.len();
    let cols = a[0].len();
    let mut weighted_gain = 0.0f64;
    let mut conditional_gain = 0.0f64;
    for recommendation in 0..rows {
        let mass: f64 = (0..cols)
            .map(|column| joint[recommendation * cols + column])
            .sum();
        for deviation in 0..rows {
            let gain: f64 = (0..cols)
                .map(|column| {
                    joint[recommendation * cols + column]
                        * (a[deviation][column] - a[recommendation][column])
                })
                .sum();
            weighted_gain = weighted_gain.max(gain);
            if mass > 0.0 {
                conditional_gain = conditional_gain.max(gain / mass);
            }
        }
    }
    for recommendation in 0..cols {
        let mass: f64 = (0..rows)
            .map(|row| joint[row * cols + recommendation])
            .sum();
        for deviation in 0..cols {
            let gain: f64 = (0..rows)
                .map(|row| {
                    joint[row * cols + recommendation]
                        * (b[row][deviation] - b[row][recommendation])
                })
                .sum();
            weighted_gain = weighted_gain.max(gain);
            if mass > 0.0 {
                conditional_gain = conditional_gain.max(gain / mass);
            }
        }
    }
    Metrics {
        mass_error: (joint.iter().sum::<f64>() - 1.0).abs(),
        negative_mass: joint.iter().map(|x| -x).fold(0.0, f64::max),
        weighted_gain,
        conditional_gain,
        objective: objective
            .iter()
            .zip(joint)
            .map(|(value, probability)| value * probability)
            .sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clarabel::solver::SolverStatus;

    fn certificate_case(id: &str, a: &[Vec<f64>], b: &[Vec<f64>], c: &[f64], upper: f64) {
        let result = solve_ce(a, b, c, 200);
        let check = metrics(a, b, c, &result.x);
        println!("{id}: status={:?} iterations={} x={:?} objective={} r_prim={} r_dual={} recomputed={check:?}",
            result.status, result.iterations, result.x, -result.obj_val, result.r_prim, result.r_dual);
        assert_eq!(result.status, SolverStatus::Solved);
        assert!(result.x.iter().all(|value| value.is_finite()));
        assert!(check.mass_error < 1e-8);
        assert!(check.negative_mass < 1e-8);
        assert!(check.weighted_gain < 1e-8);
        assert!((check.objective - upper).abs() < 1e-8);
        assert!((check.objective + result.obj_val).abs() < 1e-8);
        assert!((result.obj_val - result.obj_val_dual).abs() < 1e-8);
        assert!(result.r_prim <= 1e-8 && result.r_dual <= 1e-8);
    }

    #[test]
    fn correlation_welfare_gap() {
        certificate_case(
            "correlation_welfare_gap",
            &[vec![0., 7.], vec![2., 6.]],
            &[vec![0., 2.], vec![7., 6.]],
            &[0., 9., 9., 12.],
            10.5,
        );
    }
    #[test]
    fn dominance_zero_recommendations() {
        certificate_case(
            "dominance_zero_recommendations",
            &[vec![2., 2.], vec![1., 1.]],
            &[vec![2., 1.], vec![2., 1.]],
            &[4., 3., 3., 2.],
            4.,
        );
    }
    #[test]
    fn rectangular_two_by_three() {
        certificate_case(
            "rectangular_2x3",
            &[vec![1., -1., 0.], vec![-1., 1., 0.]],
            &[vec![-1., 1., -2.], vec![1., -1., -2.]],
            &[0., 0., -2., 0., 0., -2.],
            0.,
        );
    }
    #[test]
    fn negative_single_action() {
        certificate_case(
            "negative_single_action",
            &[vec![-3.]],
            &[vec![-4.]],
            &[-7.],
            -7.,
        );
    }
    #[test]
    fn max_iterations_is_not_a_solution() {
        let result = solve_ce(
            &[vec![0., 7.], vec![2., 6.]],
            &[vec![0., 2.], vec![7., 6.]],
            &[0., 9., 9., 12.],
            0,
        );
        println!("iteration budget: {:?}, x={:?}", result.status, result.x);
        assert_eq!(result.status, SolverStatus::MaxIterations);
        assert_ne!(result.status, SolverStatus::Solved);
    }
    #[test]
    fn nash_product_does_not_satisfy_the_objective_certificate() {
        let a = [vec![0., 7.], vec![2., 6.]];
        let b = [vec![0., 2.], vec![7., 6.]];
        let c = [0., 9., 9., 12.];
        let result = metrics(&a, &b, &c, &[1. / 9., 2. / 9., 2. / 9., 4. / 9.]);
        assert!(result.weighted_gain < 1e-14);
        assert!((result.objective - 28. / 3.).abs() < 1e-14);
        assert!(10.5 - result.objective > 1.0);
    }
    #[test]
    fn products_of_ce_marginals_break_incentives() {
        let result = metrics(
            &[vec![0., 7.], vec![2., 6.]],
            &[vec![0., 2.], vec![7., 6.]],
            &[0., 9., 9., 12.],
            &[1. / 16., 3. / 16., 3. / 16., 9. / 16.],
        );
        assert_eq!(result.weighted_gain, 3. / 16.);
        assert_eq!(result.conditional_gain, 1. / 4.);
    }
    #[test]
    fn infeasible_high_objective_is_rejected_by_incentives() {
        let result = metrics(
            &[vec![0., 7.], vec![2., 6.]],
            &[vec![0., 2.], vec![7., 6.]],
            &[0., 9., 9., 12.],
            &[0., 0., 0., 1.],
        );
        assert_eq!(result.objective, 12.);
        assert_eq!(result.weighted_gain, 1.);
    }
}
