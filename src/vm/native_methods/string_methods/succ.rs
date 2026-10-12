// The string that follows this one.

/// Ruby's `String#succ`: the rightmost alphanumeric character is bumped, and a
/// carry moves left, growing the string when the leftmost one wraps. A string
/// with no alphanumeric character bumps its last byte instead.
pub(crate) fn successor_of(text: &str) -> String {
    successor_in(text, true)
}

/// `successor_of` for a string that may not be valid text in a Unicode
/// encoding, where only ASCII letters and digits step.
pub(crate) fn successor_in(text: &str, unicode: bool) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut letters: Vec<char> = text.chars().collect();
    let alphanumeric: Vec<usize> = letters
        .iter()
        .enumerate()
        .filter(|(_, letter)| letter.is_ascii_alphanumeric() || (unicode && counts_up(**letter)))
        .map(|(index, _)| index)
        .collect();
    let mut carry = None;
    for position in alphanumeric.iter().rev() {
        match bump(letters[*position]) {
            Neighbor::Found(next) => {
                letters[*position] = next;
                return letters.into_iter().collect();
            }
            Neighbor::Wrapped { next, carried } => {
                letters[*position] = next;
                carry = Some((*position, carried));
            }
            Neighbor::Alone => {}
        }
    }
    // Every character carried, so one more is put in front of the last one
    // that did: "zz" grows into "aaa" and "99" into "100".
    match carry {
        Some((position, carried)) => {
            letters.insert(position, carried);
            letters.into_iter().collect()
        }
        None => step_last_character(letters),
    }
}

/// The successor of a string with no letter or digit that steps, which
/// counts its characters up as the bytes they stand for.
fn step_last_character(mut letters: Vec<char>) -> String {
    // A carry past the front puts one more character there.
    if letters.iter().all(|letter| (*letter as u32) < 256) {
        let mut at = letters.len();
        loop {
            if at == 0 {
                letters.insert(0, '\u{1}');
                break;
            }
            at -= 1;
            if letters[at] == '\u{ff}' {
                letters[at] = '\0';
                continue;
            }
            letters[at] = char::from_u32(letters[at] as u32 + 1).unwrap_or(letters[at]);
            break;
        }
        return letters.into_iter().collect();
    }
    let last = letters.len() - 1;
    let bumped = (letters[last] as u32).wrapping_add(1);
    if let Some(letter) = char::from_u32(bumped) {
        letters[last] = letter;
    }
    letters.into_iter().collect()
}

/// What one character of a `succ` becomes.
pub(crate) enum Neighbor {
    /// The next character of the same kind.
    Found(char),
    /// The first of its run, carrying `carried` into the place to its left.
    Wrapped { next: char, carried: char },
    /// A letter or digit standing alone in its run, which is passed over.
    Alone,
}

/// Whether `succ` counts a character up as a letter or a digit.
fn counts_up(letter: char) -> bool {
    letter.is_ascii_alphanumeric()
        || (!letter.is_ascii() && (letter.is_alphabetic() || decimal_digit(letter)))
}

/// One character of a `succ`. An ASCII letter or digit steps within `a-z`,
/// `A-Z` or `0-9`. Any other steps to the next character of its kind,
/// letter or decimal digit, passing over at most one of another kind, and
/// wraps to the first of its run otherwise, as Ruby's `enc_succ_alnum_char`
/// does.
pub(crate) fn bump(letter: char) -> Neighbor {
    match letter {
        'z' => {
            return Neighbor::Wrapped {
                next: 'a',
                carried: 'a',
            };
        }
        'Z' => {
            return Neighbor::Wrapped {
                next: 'A',
                carried: 'A',
            };
        }
        '9' => {
            return Neighbor::Wrapped {
                next: '0',
                carried: '1',
            };
        }
        other if other.is_ascii() => {
            return Neighbor::Found(char::from_u32(other as u32 + 1).unwrap_or(other));
        }
        _ => {}
    }
    let digit = decimal_digit(letter);
    let same_kind = |candidate: char| {
        candidate.len_utf8() == letter.len_utf8()
            && if digit {
                decimal_digit(candidate)
            } else {
                candidate.is_alphabetic()
            }
    };
    let code = letter as u32;
    for gap in 1..=2 {
        if let Some(candidate) = char::from_u32(code + gap)
            && same_kind(candidate)
        {
            return Neighbor::Found(candidate);
        }
    }
    let mut first = letter;
    while let Some(before) = char::from_u32(first as u32 - 1).filter(|held| same_kind(*held)) {
        first = before;
    }
    if first == letter {
        return Neighbor::Alone;
    }
    let carried = if digit {
        char::from_u32(first as u32 + 1).unwrap_or(first)
    } else {
        first
    };
    Neighbor::Wrapped {
        next: first,
        carried,
    }
}

/// Whether a character is a decimal digit in Unicode, general category Nd.
fn decimal_digit(letter: char) -> bool {
    let code = letter as u32;
    DECIMAL_DIGIT_RUNS
        .iter()
        .any(|(first, last)| (*first..=*last).contains(&code))
}

/// The runs of Unicode decimal digits, general category Nd, first and last.
const DECIMAL_DIGIT_RUNS: [(u32, u32); 72] = [
    (0x30, 0x39),
    (0x660, 0x669),
    (0x6F0, 0x6F9),
    (0x7C0, 0x7C9),
    (0x966, 0x96F),
    (0x9E6, 0x9EF),
    (0xA66, 0xA6F),
    (0xAE6, 0xAEF),
    (0xB66, 0xB6F),
    (0xBE6, 0xBEF),
    (0xC66, 0xC6F),
    (0xCE6, 0xCEF),
    (0xD66, 0xD6F),
    (0xDE6, 0xDEF),
    (0xE50, 0xE59),
    (0xED0, 0xED9),
    (0xF20, 0xF29),
    (0x1040, 0x1049),
    (0x1090, 0x1099),
    (0x17E0, 0x17E9),
    (0x1810, 0x1819),
    (0x1946, 0x194F),
    (0x19D0, 0x19D9),
    (0x1A80, 0x1A89),
    (0x1A90, 0x1A99),
    (0x1B50, 0x1B59),
    (0x1BB0, 0x1BB9),
    (0x1C40, 0x1C49),
    (0x1C50, 0x1C59),
    (0xA620, 0xA629),
    (0xA8D0, 0xA8D9),
    (0xA900, 0xA909),
    (0xA9D0, 0xA9D9),
    (0xA9F0, 0xA9F9),
    (0xAA50, 0xAA59),
    (0xABF0, 0xABF9),
    (0xFF10, 0xFF19),
    (0x104A0, 0x104A9),
    (0x10D30, 0x10D39),
    (0x10D40, 0x10D49),
    (0x11066, 0x1106F),
    (0x110F0, 0x110F9),
    (0x11136, 0x1113F),
    (0x111D0, 0x111D9),
    (0x112F0, 0x112F9),
    (0x11450, 0x11459),
    (0x114D0, 0x114D9),
    (0x11650, 0x11659),
    (0x116C0, 0x116C9),
    (0x116D0, 0x116E3),
    (0x11730, 0x11739),
    (0x118E0, 0x118E9),
    (0x11950, 0x11959),
    (0x11BF0, 0x11BF9),
    (0x11C50, 0x11C59),
    (0x11D50, 0x11D59),
    (0x11DA0, 0x11DA9),
    (0x11DE0, 0x11DE9),
    (0x11F50, 0x11F59),
    (0x16130, 0x16139),
    (0x16A60, 0x16A69),
    (0x16AC0, 0x16AC9),
    (0x16B50, 0x16B59),
    (0x16D70, 0x16D79),
    (0x1CCF0, 0x1CCF9),
    (0x1D7CE, 0x1D7FF),
    (0x1E140, 0x1E149),
    (0x1E2F0, 0x1E2F9),
    (0x1E4F0, 0x1E4F9),
    (0x1E5F1, 0x1E5FA),
    (0x1E950, 0x1E959),
    (0x1FBF0, 0x1FBF9),
];
