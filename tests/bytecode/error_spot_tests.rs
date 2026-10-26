// The spot each instruction's errors point at.

use metorex::bytecode::unit::{ErrorSpot, ErrorSpots};
use metorex::lexer::Position;

fn spot(name: &str) -> Option<ErrorSpot> {
    Some(ErrorSpot {
        start: Position::new(1, 1, 0),
        name: Some(name.to_string()),
    })
}

fn spots_of(spots: &ErrorSpots, count: usize) -> Vec<Option<ErrorSpot>> {
    (0..count).map(|at| spots[at].clone()).collect()
}

#[test]
fn each_instruction_reads_the_spot_it_was_given() {
    let mut spots = ErrorSpots::default();
    let given = [None, spot("first"), spot("first"), None, spot("second")];
    for (at, held) in given.iter().enumerate() {
        spots.set(at, held.clone());
    }
    assert_eq!(spots_of(&spots, 6), [&given[..], &[None]].concat());
}

#[test]
fn a_spot_changed_inside_a_run_leaves_the_rest_of_the_run_alone() {
    let mut spots = ErrorSpots::default();
    for at in 0..4 {
        spots.set(at, spot("call"));
    }
    spots.set(1, None);
    spots.set(2, spot("argument"));
    assert_eq!(
        spots_of(&spots, 4),
        [spot("call"), None, spot("argument"), spot("call")]
    );
}
