// String and comment lexing: quoted strings, escape sequences, interpolation.

use super::{InterpolationPart, Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Read a comment from # to end of line
    pub(super) fn read_comment(&mut self) -> String {
        let mut comment = String::new();
        // Skip the # character
        self.advance();

        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        comment.trim().to_string()
    }

    /// Pass over a `=begin` ... `=end` document at the start of a line, up to
    /// the end of the `=end` line, and answer whether there was one. One with
    /// no `=end` is left in place for the parser to refuse.
    pub(super) fn skip_embedded_document(&mut self) -> bool {
        let marks = |ahead: &std::iter::Peekable<std::str::Chars>, word: &str| {
            let mut letters = ahead.clone();
            word.chars().all(|wanted| letters.next() == Some(wanted))
                && matches!(letters.next(), None | Some(' ' | '\t' | '\r' | '\n'))
        };
        if !marks(&self.chars, "=begin") {
            return false;
        }
        let mut ahead = self.chars.clone();
        let mut length = 0;
        loop {
            // Move to the start of the next line.
            loop {
                match ahead.next() {
                    Some('\n') => {
                        length += 1;
                        break;
                    }
                    Some(_) => length += 1,
                    None => return false,
                }
            }
            if marks(&ahead, "=end") {
                while ahead.peek().is_some_and(|letter| *letter != '\n') {
                    ahead.next();
                    length += 1;
                }
                break;
            }
        }
        for _ in 0..length {
            self.advance();
        }
        true
    }

    /// Read a string literal (single or double quoted)
    pub(super) fn read_string(&mut self, quote: char) -> Result<TokenKind, String> {
        self.read_quoted(quote, false)
    }

    /// Read a backtick command literal, which interpolates the way a
    /// double-quoted string does and carries its parts for the parser to turn
    /// into a call.
    pub(super) fn read_command_string(&mut self) -> Result<TokenKind, String> {
        match self.read_quoted('`', true)? {
            TokenKind::String(text) => Ok(TokenKind::CommandString(vec![InterpolationPart::Text(
                text,
            )])),
            TokenKind::InterpolatedString(parts) => Ok(TokenKind::CommandString(parts)),
            // A command whose escapes name bytes runs those bytes, so the
            // text stands for them rather than for characters.
            TokenKind::ByteString(text) => Ok(TokenKind::ByteCommandString(text)),
            other => Ok(other),
        }
    }

    fn read_quoted(&mut self, quote: char, command: bool) -> Result<TokenKind, String> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        // A run of numeric escapes that spells no text leaves the literal
        // standing for bytes rather than for characters.
        let mut holds_bytes = false;
        // Only double-quoted strings and backticks support interpolation.
        let has_interpolation = quote == '"' || command;

        // Skip the opening quote
        self.advance();

        loop {
            match self.peek() {
                None => {
                    return Err(format!(
                        "Unterminated string starting at line {}",
                        self.line
                    ));
                }
                // A quoted string may run across lines, so a raw newline is
                // content rather than the end of the literal.
                Some('\n') => {
                    current_text.push('\n');
                    self.advance();
                }
                Some(ch) if ch == quote => {
                    // Found closing quote
                    self.advance();

                    // If we have interpolation parts, return an interpolated string
                    if has_interpolation && !parts.is_empty() {
                        if !current_text.is_empty() {
                            parts.push(InterpolationPart::Text(current_text));
                        }
                        return Ok(TokenKind::InterpolatedString(parts));
                    }
                    // Every character of a literal that holds bytes stands for
                    // one: an escaped byte for itself, and any other character
                    // for the bytes that spell it.
                    if holds_bytes {
                        current_text = byte_text(&current_text);
                    }
                    if self.frozen_literals && !command {
                        // The source asked for its literals to be frozen, and
                        // one written in escapes still stands for its bytes,
                        // in bytes where the source is written in them.
                        if holds_bytes && self.binary_source {
                            return Ok(TokenKind::FrozenBinaryString(current_text));
                        }
                        if holds_bytes {
                            return Ok(TokenKind::FrozenByteString(current_text));
                        }
                        return Ok(TokenKind::FrozenString(current_text));
                    } else if self.mutable_literals && !command && !holds_bytes {
                        // The source asked outright for literals that change,
                        // so one carries no notice that it will be frozen.
                        return Ok(TokenKind::MutableString(current_text));
                    } else if holds_bytes {
                        // A source written in bytes spells its literals in
                        // bytes; anywhere else the characters stand for the
                        // bytes the escapes named, in the source's encoding.
                        if self.binary_source {
                            return Ok(TokenKind::BinaryString(current_text));
                        }
                        return Ok(TokenKind::ByteString(current_text));
                    } else {
                        return Ok(TokenKind::String(current_text));
                    }
                }
                Some('\\') => {
                    // Handle escape sequences
                    self.advance();
                    // A single-quoted string reads only `\\` and `\'` as
                    // escapes, so every other backslash stands for itself.
                    if quote == '\'' {
                        match self.peek() {
                            Some(next @ ('\\' | '\'')) => {
                                current_text.push(next);
                                self.advance();
                            }
                            _ => current_text.push('\\'),
                        }
                        continue;
                    }
                    match self.peek() {
                        Some('n') => {
                            current_text.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            current_text.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            current_text.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            current_text.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            current_text.push('"');
                            self.advance();
                        }
                        Some('\'') => {
                            current_text.push('\'');
                            self.advance();
                        }
                        Some('#') => {
                            // Escaped hash - allows literal #{
                            current_text.push('#');
                            self.advance();
                        }
                        Some('e') => {
                            // ESC character (0x1B) for ANSI escape sequences
                            current_text.push('\x1B');
                            self.advance();
                        }
                        // An octal escape runs to three digits, so `\000` is
                        // one NUL rather than a NUL followed by two zeros. A
                        // run of them names bytes, which together may spell
                        // one character.
                        Some(digit) if ('0'..='7').contains(&digit) => {
                            let mut bytes = Vec::new();
                            loop {
                                let mut digits = String::new();
                                while digits.len() < 3
                                    && self.peek().is_some_and(|next| ('0'..='7').contains(&next))
                                {
                                    digits.push(self.peek().expect("a digit was seen"));
                                    self.advance();
                                }
                                bytes.push(u32::from_str_radix(&digits, 8).unwrap_or(0) as u8);
                                if self.peek() != Some('\\') {
                                    break;
                                }
                                self.advance();
                                if !self.peek().is_some_and(|next| ('0'..='7').contains(&next)) {
                                    // The backslash opened some other escape,
                                    // so it is handed back to the main loop.
                                    self.push_back('\\');
                                    break;
                                }
                            }
                            match binary_run(self.binary_source, &bytes) {
                                Ok(text) => current_text.push_str(&text),
                                Err(_) => {
                                    holds_bytes = true;
                                    push_escaped_bytes(&mut current_text, &bytes);
                                }
                            }
                        }
                        Some('s') => {
                            current_text.push(' ');
                            self.advance();
                        }
                        Some('a') => {
                            current_text.push('\u{7}');
                            self.advance();
                        }
                        Some('b') => {
                            current_text.push('\u{8}');
                            self.advance();
                        }
                        Some('f') => {
                            current_text.push('\u{c}');
                            self.advance();
                        }
                        Some('v') => {
                            current_text.push('\u{b}');
                            self.advance();
                        }
                        // `\xNN` names a byte and `\uXXXX` or `\u{...}` a
                        // code point, which is how a spec writes a character
                        // it cannot type.
                        Some('x') => {
                            // A run of `\xNN` escapes names bytes, which
                            // together may spell one character.
                            let mut bytes = Vec::new();
                            loop {
                                self.advance();
                                let mut digits = String::new();
                                while digits.len() < 2
                                    && self.peek().is_some_and(|digit| digit.is_ascii_hexdigit())
                                {
                                    digits.push(self.peek().expect("a digit was seen"));
                                    self.advance();
                                }
                                match u8::from_str_radix(&digits, 16) {
                                    Ok(byte) => bytes.push(byte),
                                    Err(_) => {
                                        current_text.push_str("\\x");
                                        current_text.push_str(&digits);
                                    }
                                }
                                if self.peek() != Some('\\') {
                                    break;
                                }
                                self.advance();
                                if self.peek() != Some('x') {
                                    // The backslash opened some other escape,
                                    // so it is handed back to the main loop.
                                    self.push_back('\\');
                                    break;
                                }
                            }
                            match binary_run(self.binary_source, &bytes) {
                                Ok(text) => current_text.push_str(&text),
                                Err(_) => {
                                    holds_bytes = true;
                                    push_escaped_bytes(&mut current_text, &bytes);
                                }
                            }
                        }
                        // `\cX`, `\C-X`, and `\M-X` name a control or meta
                        // character by the one that follows, and `\M-\C-X`
                        // names both at once.
                        Some('c') | Some('C') | Some('M') => match self.read_control_escape() {
                            Some(byte) if byte.is_ascii() => {
                                current_text.push(byte as char);
                            }
                            Some(byte) => {
                                holds_bytes = true;
                                push_escaped_bytes(&mut current_text, &[byte]);
                            }
                            None => {
                                if quote == '\'' {
                                    current_text.push('\\');
                                }
                                if let Some(letter) = self.peek() {
                                    current_text.push(letter);
                                    self.advance();
                                }
                            }
                        },
                        Some('u') => {
                            self.advance();
                            self.read_unicode_escape(&mut current_text);
                        }
                        Some(ch) => {
                            // A backslash before something that opens no escape
                            // stands for the character alone, which is how
                            // `"a\{b"` names three characters. A single-quoted
                            // string keeps the backslash, since only `\'` and
                            // `\\` mean anything inside one.
                            if quote == '\'' {
                                current_text.push('\\');
                            }
                            current_text.push(ch);
                            self.advance();
                        }
                        None => {
                            return Err(format!(
                                "Unterminated string starting at line {}",
                                self.line
                            ));
                        }
                    }
                }
                Some('#') if has_interpolation => {
                    // Check if this is the start of interpolation (#{)
                    self.advance();
                    if self.peek() == Some('{') {
                        // Start of interpolation
                        let hole_line = self.line;
                        self.advance();

                        // Save current text as a part
                        if !current_text.is_empty() {
                            parts.push(InterpolationPart::Text(current_text.clone()));
                            current_text.clear();
                        }

                        // Read the expression until we find }
                        let mut expr = String::new();
                        let mut depth = 1; // Track nested braces

                        loop {
                            match self.peek() {
                                None => {
                                    return Err(format!(
                                        "Unterminated interpolation starting at line {}",
                                        self.line
                                    ));
                                }
                                Some('{') => {
                                    depth += 1;
                                    expr.push('{');
                                    self.advance();
                                }
                                Some('}') => {
                                    depth -= 1;
                                    if depth == 0 {
                                        self.advance();
                                        parts.push(InterpolationPart::Expression(expr, hole_line));
                                        break;
                                    } else {
                                        expr.push('}');
                                        self.advance();
                                    }
                                }
                                // A string written inside the expression is
                                // copied whole, so a brace it holds does not
                                // count toward the ones the expression opens.
                                Some(inner @ ('"' | '\'' | '`')) if opens_a_string(&expr) => {
                                    self.copy_quoted_in_interpolation(inner, &mut expr)?;
                                }
                                Some('%') if self.copy_percent_literal_in_interpolation(&mut expr)? => {}
                                Some('<') if self.copy_heredoc_in_interpolation(&mut expr) => {}
                                Some('/') if opens_a_regexp(&expr, &self.peek_several(2)) => {
                                    self.copy_quoted_in_interpolation('/', &mut expr)?;
                                }
                                Some(ch) => {
                                    expr.push(ch);
                                    self.advance();
                                }
                            }
                        }
                    } else if let Some(named) = self.read_short_interpolation() {
                        // `#@name`, `#@@name`, and `#$name` interpolate that
                        // variable without braces around it.
                        if !current_text.is_empty() {
                            parts.push(InterpolationPart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        parts.push(InterpolationPart::Expression(named, self.line));
                    } else {
                        // Not interpolation, just a # character
                        current_text.push('#');
                    }
                }
                Some(ch) => {
                    current_text.push(ch);
                    self.advance();
                }
            }
        }
    }
}

/// The text a run of numeric escapes spells. In a source written in bytes
/// each byte stands alone, and elsewhere bytes that spell a character in
/// UTF-8 read back as that character.
/// Add bytes an escape named: an ASCII one as its character, and any other
/// as the marker the file loader gives a byte, so it stays apart from a
/// character that happens to share its number.
pub(super) fn push_escaped_bytes(text: &mut String, bytes: &[u8]) {
    for byte in bytes {
        if byte.is_ascii() {
            text.push(*byte as char);
        } else {
            text.push(super::escaped_bytes::escaped_byte(*byte));
        }
    }
}

/// A literal's text read as bytes, one character to each: a marked byte for
/// itself, and any other character for the bytes that spell it in UTF-8.
pub(super) fn byte_text(text: &str) -> String {
    let bytes = super::escaped_bytes::escaped_source_bytes(text)
        .unwrap_or_else(|| text.as_bytes().to_vec());
    bytes.iter().map(|byte| *byte as char).collect()
}

pub(super) fn binary_run(binary_source: bool, bytes: &[u8]) -> Result<String, ()> {
    if binary_source {
        return Err(());
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| ())
}

/// How many of the characters at the start of `rest` name the variable a `#`
/// interpolates without braces: `@name`, `@@name`, `$name`, a special global
/// such as `$1` or `$~`, or one such as `$-w`. Zero where they name none, so
/// `"#@ "` stands as it reads.
pub(super) fn short_variable_length(rest: &[char]) -> usize {
    let starts_a_name = |letter: &char| letter.is_alphabetic() || *letter == '_';
    let continues_a_name = |letter: &char| letter.is_alphanumeric() || *letter == '_';
    let name_from = |start: usize| -> usize {
        if !rest.get(start).is_some_and(starts_a_name) {
            return 0;
        }
        start
            + rest[start..]
                .iter()
                .take_while(|letter| continues_a_name(letter))
                .count()
    };
    match rest.first() {
        Some('@') => name_from(if rest.get(1) == Some(&'@') { 2 } else { 1 }),
        Some('$') => match rest.get(1) {
            Some(digit) if digit.is_ascii_digit() => {
                1 + rest[1..]
                    .iter()
                    .take_while(|letter| letter.is_ascii_digit())
                    .count()
            }
            Some(mark) if "~*$?!@/\\;,.=:<>\"&`'+".contains(*mark) => 2,
            Some('-') if rest.get(2).is_some_and(continues_a_name) => 3,
            _ => name_from(1),
        },
        _ => 0,
    }
}

/// Whether a quote that follows `copied` opens a string, rather than ending
/// a global's name such as `$'`, `$"`, or `` $` ``.
fn opens_a_string(copied: &str) -> bool {
    !copied.ends_with('$')
}

/// Whether a `/` that follows `copied` opens a regexp rather than dividing:
/// where it starts the expression or follows an operator or an opening
/// bracket, or follows a space and is not followed by one, as in
/// `gsub /x/`. `slash_and_next` holds the `/` and the character after it.
fn opens_a_regexp(copied: &str, slash_and_next: &[char]) -> bool {
    let before = copied.trim_end();
    match before.chars().last() {
        None => true,
        Some(last) if "({[,=|&!;~<>?:+-*%^".contains(last) => true,
        Some(_) => {
            before.len() < copied.len()
                && !slash_and_next.get(1).is_some_and(|next| next.is_whitespace() || *next == '=')
        }
    }
}

impl<'a> Lexer<'a> {
    /// Copy a string or regexp literal written inside an interpolation into
    /// `expr`, from its opening quote to its closing one. An escaped
    /// character is copied with its backslash, and a `#{` in any but a
    /// single-quoted string copies the expression it opens, braces and
    /// strings and all.
    fn copy_quoted_in_interpolation(&mut self, quote: char, expr: &mut String) -> Result<(), String> {
        let interpolates = quote != '\'';
        expr.push(quote);
        self.advance();
        loop {
            match self.peek() {
                None => {
                    return Err(format!("Unterminated string starting at line {}", self.line));
                }
                Some('\\') => {
                    expr.push('\\');
                    self.advance();
                    if let Some(escaped) = self.peek() {
                        expr.push(escaped);
                        self.advance();
                    }
                }
                Some(ch) if ch == quote => {
                    expr.push(ch);
                    self.advance();
                    return Ok(());
                }
                Some('#') if interpolates => {
                    expr.push('#');
                    self.advance();
                    if self.peek() == Some('{') {
                        expr.push('{');
                        self.advance();
                        self.copy_interpolated_expression(expr)?;
                    }
                }
                Some(ch) => {
                    expr.push(ch);
                    self.advance();
                }
            }
        }
    }

    /// Read a heredoc opened inside an interpolation, as `"#{<<~RUBY}"`
    /// does, whose body is on the lines after the string's, and copy it into
    /// `expr` as a double-quoted literal. Answers whether one was there.
    fn copy_heredoc_in_interpolation(&mut self, expr: &mut String) -> bool {
        if !matches!(self.peek_several(3).as_slice(), ['<', '<', '~' | '-']) {
            return false;
        }
        let value_starts_here = match expr.trim_end().chars().last() {
            None => true,
            Some(last) => !(last.is_alphanumeric() || matches!(last, '_' | ')' | ']' | '}' | '"' | '\'')),
        };
        if !value_starts_here {
            return false;
        }
        let saved = (
            self.chars.clone(),
            self.prepend.clone(),
            self.line,
            self.column,
            self.offset,
        );
        self.advance();
        self.advance();
        let literal = self.try_read_heredoc().and_then(|read| double_quoted_literal(&read));
        match literal {
            Some(literal) => {
                expr.push_str(&literal);
                true
            }
            None => {
                (self.chars, self.prepend, self.line, self.column, self.offset) = saved;
                false
            }
        }
    }

    /// Copy a percent literal written inside an interpolation, such as
    /// `%q{', "'", '}`, into `expr` whole, so a quote or brace it holds is
    /// not read as one the expression writes. Answers whether one was there:
    /// a `%` is a literal only where a value starts, so `a % b` is left to
    /// be copied as the operator.
    fn copy_percent_literal_in_interpolation(&mut self, expr: &mut String) -> Result<bool, String> {
        let ahead = self.peek_several(3);
        let (prefix, opening) = match ahead.as_slice() {
            ['%', kind @ ('q' | 'Q' | 'w' | 'W' | 'i' | 'I' | 's' | 'r' | 'x'), opening, ..]
                if !opening.is_alphanumeric() && !opening.is_whitespace() =>
            {
                (format!("%{kind}"), *opening)
            }
            ['%', opening @ ('{' | '(' | '[' | '<' | '|' | '!'), ..] => ("%".to_string(), *opening),
            _ => return Ok(false),
        };
        let value_starts_here = match expr.trim_end().chars().last() {
            None => true,
            Some(last) => !(last.is_alphanumeric() || matches!(last, '_' | ')' | ']' | '}' | '"' | '\'')),
        };
        if !value_starts_here {
            return Ok(false);
        }
        let closing = match opening {
            '{' => '}',
            '(' => ')',
            '[' => ']',
            '<' => '>',
            other => other,
        };
        for _ in 0..prefix.chars().count() + 1 {
            self.advance();
        }
        expr.push_str(&prefix);
        expr.push(opening);
        let mut depth = 1;
        loop {
            match self.advance() {
                None => return Err(format!("Unterminated string starting at line {}", self.line)),
                Some('\\') => {
                    expr.push('\\');
                    if let Some(escaped) = self.advance() {
                        expr.push(escaped);
                    }
                }
                Some(ch) if ch == closing => {
                    expr.push(ch);
                    depth -= 1;
                    if depth == 0 {
                        return Ok(true);
                    }
                }
                Some(ch) => {
                    if ch == opening && opening != closing {
                        depth += 1;
                    }
                    expr.push(ch);
                }
            }
        }
    }

    /// Copy an interpolated expression into `expr` up to and including the
    /// `}` that closes it, the `#{` having been copied already.
    fn copy_interpolated_expression(&mut self, expr: &mut String) -> Result<(), String> {
        let mut depth = 1;
        loop {
            match self.peek() {
                None => {
                    return Err(format!(
                        "Unterminated interpolation starting at line {}",
                        self.line
                    ));
                }
                Some('{') => depth += 1,
                Some('}') => {
                    depth -= 1;
                    if depth == 0 {
                        expr.push('}');
                        self.advance();
                        return Ok(());
                    }
                }
                Some(inner @ ('"' | '\'' | '`')) if opens_a_string(expr) => {
                    self.copy_quoted_in_interpolation(inner, expr)?;
                    continue;
                }
                Some('%') if self.copy_percent_literal_in_interpolation(expr)? => continue,
                Some('/') if opens_a_regexp(expr, &self.peek_several(2)) => {
                    self.copy_quoted_in_interpolation('/', expr)?;
                    continue;
                }
                Some(_) => {}
            }
            let ch = self.peek().expect("a character was seen");
            expr.push(ch);
            self.advance();
        }
    }

    /// The variable a `#` interpolates without braces, read from just past
    /// the `#`. Nothing is consumed where what follows names no variable.
    pub(super) fn read_short_interpolation(&mut self) -> Option<String> {
        const LONGEST_NAME: usize = 1024;
        let upcoming: Vec<char> = self
            .prepend
            .iter()
            .rev()
            .copied()
            .chain(self.chars.clone())
            .take(LONGEST_NAME)
            .collect();
        let length = short_variable_length(&upcoming);
        if length == 0 {
            return None;
        }
        for _ in 0..length {
            self.advance();
        }
        Some(upcoming[..length].iter().collect())
    }
}

impl<'a> Lexer<'a> {
    /// The byte a control or meta escape names. The cursor sits on the letter
    /// that opened it: `c` for `\cX`, `C` for `\C-X`, or `M` for `\M-X`.
    /// Nothing is consumed where what follows spells no such escape.
    /// The characters a `\u` escape names, read after the `u`: four hex
    /// digits, or code points in braces separated by spaces. A code point
    /// that names no character is kept as written.
    pub(super) fn read_unicode_escape(&mut self, text: &mut String) {
        let mut points = Vec::new();
        if self.peek() == Some('{') {
            self.advance();
            let mut digits = String::new();
            while let Some(letter) = self.peek() {
                if letter == '}' {
                    self.advance();
                    break;
                }
                if letter == ' ' {
                    points.push(std::mem::take(&mut digits));
                } else {
                    digits.push(letter);
                }
                self.advance();
            }
            points.push(digits);
            points.retain(|point| !point.is_empty());
        } else {
            let mut digits = String::new();
            while digits.len() < 4 && self.peek().is_some_and(|digit| digit.is_ascii_hexdigit()) {
                digits.push(self.peek().expect("a digit was seen"));
                self.advance();
            }
            points.push(digits);
        }
        for point in points {
            match u32::from_str_radix(&point, 16).ok().and_then(char::from_u32) {
                Some(letter) => text.push(letter),
                None => {
                    text.push('\\');
                    text.push('u');
                    text.push_str(&point);
                }
            }
        }
    }

    pub(super) fn read_control_escape(&mut self) -> Option<u8> {
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let rewind = |held: &mut Self| {
            held.chars = saved_chars.clone();
            held.prepend = saved_prepend.clone();
            held.line = saved_line;
            held.column = saved_column;
            held.offset = saved_offset;
        };
        let opener = self.peek()?;
        self.advance();
        // `\C-` and `\M-` are written with the hyphen, while `\c` is not.
        if opener != 'c' {
            if self.peek() != Some('-') {
                rewind(self);
                return None;
            }
            self.advance();
        }
        // The escapes stack, so `\M-\C-z` is meta over control.
        let held = match self.peek() {
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('c') | Some('C') | Some('M') => self.read_control_escape()?,
                    Some(letter) if letter.is_ascii() => {
                        self.advance();
                        letter as u8
                    }
                    _ => {
                        rewind(self);
                        return None;
                    }
                }
            }
            Some(letter) if letter.is_ascii() => {
                self.advance();
                letter as u8
            }
            _ => {
                rewind(self);
                return None;
            }
        };
        Some(match opener {
            // Meta sets the high bit, and control keeps the low six.
            'M' => held | 0x80,
            _ => held & 0x9f,
        })
    }
}

/// A string read from the source written back as a double-quoted literal
/// that reads as the same string, its interpolations kept as they were.
fn double_quoted_literal(read: &TokenKind) -> Option<String> {
    let mut literal = String::from("\"");
    let write_text = |text: &str, literal: &mut String| {
        for character in text.chars() {
            match character {
                '\\' => literal.push_str("\\\\"),
                '"' => literal.push_str("\\\""),
                '#' => literal.push_str("\\#"),
                '\n' => literal.push_str("\\n"),
                '\t' => literal.push_str("\\t"),
                '\r' => literal.push_str("\\r"),
                other => literal.push(other),
            }
        }
    };
    match read {
        TokenKind::String(text) => write_text(text, &mut literal),
        TokenKind::InterpolatedString(parts) => {
            for part in parts {
                match part {
                    InterpolationPart::Text(text) => write_text(text, &mut literal),
                    InterpolationPart::Expression(code, _) => {
                        literal.push_str("#{");
                        literal.push_str(code);
                        literal.push('}');
                    }
                }
            }
        }
        _ => return None,
    }
    literal.push('"');
    Some(literal)
}
