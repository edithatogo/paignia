//! Compare both enclosure methods at the same finest strategy grid.

use std::hint::black_box;
use std::time::{Duration, Instant};

use paignia_solve::{enclose_credal_nash, enclose_credal_nash_segment};

#[path = "../tests/support/credal_segment_cases.rs"]
mod cases;

fn median(mut operation: impl FnMut()) -> Duration {
    const RUNS: usize = 21;
    operation();
    let mut measurements = [Duration::ZERO; RUNS];
    for duration in &mut measurements {
        let start = Instant::now();
        operation();
        *duration = start.elapsed();
    }
    measurements.sort_unstable();
    measurements[RUNS / 2]
}

fn main() {
    let depth = 6;
    let divisions = 1 << depth;
    println!("case,grid_cells,grid_kept,adaptive_examined,adaptive_kept,grid_us,adaptive_us");
    for (name, vertices) in [
        (
            "shared-weight-conflict",
            [cases::dominance(1, -3), cases::dominance(-3, 1)],
        ),
        ("matching-pennies", [cases::pennies(1); 2]),
        ("interior-all-tie", [cases::pennies(1), cases::pennies(-1)]),
    ] {
        let grid = || enclose_credal_nash(black_box(&vertices), divisions).unwrap();
        let adaptive = || enclose_credal_nash_segment(black_box(&vertices), depth).unwrap();
        let grid_result = grid();
        let adaptive_result = adaptive();
        let kept = adaptive_result
            .leaves
            .iter()
            .filter(|cell| cell.witness.is_some())
            .count();
        let grid_time = median(|| {
            black_box(grid());
        });
        let adaptive_time = median(|| {
            black_box(adaptive());
        });
        println!(
            "{name},{},{},{},{kept},{},{}",
            grid_result.total_cells,
            grid_result.retained_cells,
            adaptive_result.examined_cells,
            grid_time.as_micros(),
            adaptive_time.as_micros(),
        );
    }
}
