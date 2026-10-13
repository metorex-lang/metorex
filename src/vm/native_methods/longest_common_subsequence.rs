// The longest common subsequence Rex's differ is built on, computed the way
// diff-lcs computes it, so a diff Rex prints pairs the same lines RSpec's
// does when more than one pairing is equally long.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use std::collections::HashMap;

/// For each position in `old`, the position in `new` it is paired with, or
/// `None`. The list ends at the last paired position of `old`, as diff-lcs's
/// does, since its traversal treats the positions past the end differently.
pub(crate) fn matches(old: &[String], new: &[String]) -> Vec<Option<usize>> {
    let mut vector: Vec<Option<usize>> = Vec::new();
    let pair = |vector: &mut Vec<Option<usize>>, at: usize, with: usize| {
        if vector.len() <= at {
            vector.resize(at + 1, None);
        }
        vector[at] = Some(with);
    };

    // The common lines at the start, then the common lines at the end.
    let mut old_start = 0;
    let mut new_start = 0;
    let mut old_end = old.len();
    let mut new_end = new.len();
    while old_start < old_end && new_start < new_end && old[old_start] == new[new_start] {
        pair(&mut vector, old_start, new_start);
        old_start += 1;
        new_start += 1;
    }
    while old_start < old_end && new_start < new_end && old[old_end - 1] == new[new_end - 1] {
        pair(&mut vector, old_end - 1, new_end - 1);
        old_end -= 1;
        new_end -= 1;
    }

    let mut positions_in_new: HashMap<&str, Vec<usize>> = HashMap::new();
    for (at, line) in new.iter().enumerate().take(new_end).skip(new_start) {
        positions_in_new.entry(line.as_str()).or_default().push(at);
    }

    // `thresholds[k]` is the smallest position in `new` that ends a common
    // subsequence of length k + 1, and `links[k]` the chain that reaches it.
    let mut thresholds: Vec<usize> = Vec::new();
    let mut chains: Vec<(Option<usize>, usize, usize)> = Vec::new();
    let mut links: Vec<usize> = Vec::new();
    for (old_at, line) in old.iter().enumerate().take(old_end).skip(old_start) {
        let Some(found) = positions_in_new.get(line.as_str()) else {
            continue;
        };
        let mut length: Option<usize> = None;
        for &new_at in found.iter().rev() {
            match length {
                Some(k) if k > 0 && thresholds[k] > new_at && thresholds[k - 1] < new_at => {
                    thresholds[k] = new_at;
                }
                _ => length = replace_next_larger(&mut thresholds, new_at, length),
            }
            if let Some(k) = length {
                let previous = if k > 0 { Some(links[k - 1]) } else { None };
                chains.push((previous, old_at, new_at));
                let link = chains.len() - 1;
                if k == links.len() {
                    links.push(link);
                } else {
                    links[k] = link;
                }
            }
        }
    }

    if let Some(&last) = links.get(thresholds.len().wrapping_sub(1)) {
        let mut link = Some(last);
        while let Some(at) = link {
            let (previous, old_at, new_at) = chains[at];
            pair(&mut vector, old_at, new_at);
            link = previous;
        }
    }
    vector
}

/// Puts `value` where it keeps `thresholds` sorted, replacing the next
/// larger value or appending it, and answers where it went. A value already
/// there answers `None`. The search ends at `last_index` when one is given.
fn replace_next_larger(
    thresholds: &mut Vec<usize>,
    value: usize,
    last_index: Option<usize>,
) -> Option<usize> {
    if thresholds.last().is_none_or(|&last| value > last) {
        thresholds.push(value);
        return Some(thresholds.len() - 1);
    }
    let mut first = 0isize;
    let mut last = last_index.unwrap_or(thresholds.len() - 1) as isize;
    while first <= last {
        let middle = ((first + last) >> 1) as usize;
        let found = thresholds[middle];
        if value == found {
            return None;
        }
        if value > found {
            first = middle as isize + 1;
        } else {
            last = middle as isize - 1;
        }
    }
    thresholds[first as usize] = value;
    Some(first as usize)
}

impl VirtualMachine {
    /// `Rex::Diff.__matches__(old_lines, new_lines)`: the pairing `matches`
    /// answers, as an Array of Integers and nils.
    pub(crate) fn longest_common_subsequence_matches(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let lines = |argument: Option<&Object>| -> Option<Vec<String>> {
            let Some(Object::Array(elements)) = argument else {
                return None;
            };
            elements
                .borrow()
                .iter()
                .map(|element| match element {
                    Object::String(text) => Some(text.as_str().to_string()),
                    _ => None,
                })
                .collect()
        };
        let (Some(old), Some(new)) = (lines(arguments.first()), lines(arguments.get(1))) else {
            return Err(simple_exception(
                "TypeError",
                "Rex::Diff.__matches__ takes two Arrays of Strings",
                position,
            ));
        };
        Ok(Object::array(
            matches(&old, &new)
                .into_iter()
                .map(|paired| paired.map_or(Object::Nil, |at| Object::Int(at as i64)))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::matches;

    fn lines(text: &str) -> Vec<String> {
        text.split(' ').map(str::to_string).collect()
    }

    #[test]
    fn identical_sequences_pair_every_line() {
        assert_eq!(
            matches(&lines("a b c"), &lines("a b c")),
            vec![Some(0), Some(1), Some(2)]
        );
    }

    #[test]
    fn the_pairing_ends_at_the_last_paired_line() {
        assert_eq!(
            matches(&lines("a b x"), &lines("a b")),
            vec![Some(0), Some(1)]
        );
    }

    #[test]
    fn a_middle_change_pairs_the_lines_around_it() {
        assert_eq!(
            matches(&lines("a b c d e"), &lines("a x c y e")),
            vec![Some(0), None, Some(2), None, Some(4)]
        );
    }

    #[test]
    fn of_two_equally_long_pairings_the_one_diff_lcs_picks_is_kept() {
        assert_eq!(
            matches(&lines("a b c"), &lines("b a c")),
            vec![None, Some(0), Some(2)]
        );
    }

    #[test]
    fn nothing_in_common_pairs_nothing() {
        assert_eq!(
            matches(&lines("a b"), &lines("c d")),
            Vec::<Option<usize>>::new()
        );
    }
}
