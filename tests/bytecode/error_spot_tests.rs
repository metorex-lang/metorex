// The spot each instruction's errors point at.

use metorex::bytecode::unit::{ErrorSpot, ErrorSpots};
use metorex::lexer::Position;

fn spot(spots: &mut ErrorSpots, name: &str) -> Option<ErrorSpot> {
    Some(spots.spot(Position::new(1, 1, 0), Some(name)))
}

fn spots_of(spots: &ErrorSpots, count: usize) -> Vec<Option<ErrorSpot>> {
    (0..count).map(|at| spots.get(at)).collect()
}

#[test]
fn each_instruction_reads_the_spot_it_was_given() {
    let mut spots = ErrorSpots::default();
    let (first, second) = (spot(&mut spots, "first"), spot(&mut spots, "second"));
    let given = [None, first, first, None, second];
    for (at, held) in given.iter().enumerate() {
        spots.set(at, *held);
    }
    assert_eq!(spots_of(&spots, 6), [&given[..], &[None]].concat());
}

#[test]
fn instructions_between_runs_of_one_spot_have_none() {
    let mut spots = ErrorSpots::default();
    let call = spot(&mut spots, "call");
    let given = [call, call, None, call];
    for (at, held) in given.iter().enumerate() {
        spots.set(at, *held);
    }
    assert_eq!(spots_of(&spots, 4), given);
}

#[test]
fn two_spots_naming_one_name_each_read_it() {
    let mut spots = ErrorSpots::default();
    let first = spots.spot(Position::new(1, 1, 0), Some("held"));
    let second = spots.spot(Position::new(2, 1, 9), Some("held"));

    assert_eq!(
        (
            spots.name_of(&first),
            spots.name_of(&second),
            first == second
        ),
        (Some("held"), Some("held"), false)
    );
}

#[test]
#[should_panic(expected = "spots are given in instruction order")]
fn a_spot_is_given_after_the_ones_before_it() {
    let mut spots = ErrorSpots::default();
    spots.set(1, None);
}

#[test]
fn a_position_reads_back_as_it_was_held_with_whether_it_is_the_preludes() {
    let mut positions = metorex::bytecode::unit::Positions::default();
    let mut prelude = Position::new(7, 3, 1_000_000);
    prelude.prelude = true;
    positions.push(Position::new(2, 5, 40));
    positions.push(prelude);

    assert_eq!(
        (positions.at(0), positions.at(1)),
        (Position::new(2, 5, 40), prelude)
    );
}

#[test]
fn a_statement_start_entry_takes_40_bytes() {
    assert_eq!(
        std::mem::size_of::<(u32, Option<metorex::bytecode::unit::StatementStart>)>(),
        40
    );
}
