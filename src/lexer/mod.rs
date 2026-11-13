// Lexer module for tokenizing Metorex source code.
//
// Implementation is split across files by responsibility:
//   - cursor:      character cursor primitives (advance, peek, position)
//   - identifiers: identifiers, keywords, instance/class/global vars
//   - numbers:     integer and float literals
//   - strings:     quoted strings, escape sequences, interpolation, comments
//   - regex_lit:   `/.../` regex literals and slash-vs-regex disambiguation
//   - percent:     `%`-prefixed literals (`%r`, `%w`, `%Q`, `%[...]`, etc.)
//   - dispatch:    the main `next_token_inner` switch over leading characters

pub mod token;

mod cursor;
mod dispatch;
pub mod escaped_bytes;
mod heredoc;
pub(crate) use heredoc::split_pattern_interpolation_parts;
mod identifiers;
mod numbers;
mod percent;
mod regex_lit;
mod strings;

pub use token::{InterpolationPart, Position, Token, TokenKind};

use std::iter::Peekable;
use std::str::Chars;

/// How far a `def`'s parameter list has been read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DefParameters {
    /// No `def` is waiting for its parameters.
    Idle,
    /// A `def` was read, and its name may be followed by `(`.
    AfterDef,
    /// The parameter list is open, with this many parentheses unclosed.
    Open(usize),
}

/// The lexer converts source code into a stream of tokens
pub struct Lexer<'a> {
    /// Peekable iterator over the characters
    pub(super) chars: Peekable<Chars<'a>>,
    /// Characters injected back into the stream (consumed before `chars`).
    /// Stored in reverse order so `pop` yields the next char.
    pub(super) prepend: Vec<char>,
    /// Current position in the source
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) offset: usize,
    /// Last significant token kind (for regex vs division disambiguation)
    pub(super) prev_significant: Option<TokenKind>,
    /// Source offset just past the last significant token, which tells a
    /// symbol naming an operator (`:/`) from a colon that happens to be
    /// followed by one (`condition ? a : /re/`).
    pub(super) prev_significant_end: usize,
    /// Line number to restore once `prepend` drains. Heredoc lexing rewinds
    /// `line` to the opener's line while the rest of that line is re-lexed
    /// from `prepend`; this puts the counter back at the line following the
    /// heredoc terminator afterwards.
    pub(super) restore_line: Option<usize>,
    /// Where the source stream's offset stands while an injected heredoc
    /// rest-of-line is lexed at the offsets it was written at.
    pub(super) restore_offset: Option<usize>,
    /// Whether this lexer is reading the core library rather than a program.
    pub(super) prelude: bool,
    /// Whether the source says it is written in bytes rather than in text. A
    /// run of numeric escapes then names those bytes one by one instead of
    /// spelling a character between them.
    pub(super) binary_source: bool,
    /// Whether the source asked for its string literals to be frozen.
    pub(super) frozen_literals: bool,
    /// Whether the source asked outright for its string literals to stay
    /// mutable. A source that says nothing either way gets the literals Ruby
    /// hands back with notice that a later release will freeze them.
    pub(super) mutable_literals: bool,
    /// The encoding the source says it is written in, which is what
    /// `__ENCODING__` answers where it is written.
    pub(super) source_encoding: String,
    /// Names written where a local variable is bound or read back: before
    /// `=`, a compound assignment, `,`, `|`, `)` or `in`. A `/` after one of
    /// these divides, where after any other name it may open a pattern
    /// handed to a method called without parentheses.
    pub(super) local_names: std::collections::HashSet<String>,
    /// How many method names an `alias` has yet to be given, where a `/`
    /// names the division method rather than opening a pattern.
    pub(super) alias_names_left: u8,
    /// How far a `def`'s parameter list has been read, so the `)` closing it
    /// can be told from one closing a value.
    pub(super) def_parameters: DefParameters,
    /// Whether the last significant token was the `)` closing a `def`'s
    /// parameter list, after which a `/` opens the body with a pattern.
    pub(super) closed_def_parameters: bool,
    /// Whether the last significant token was a colon written against the
    /// name before it, which makes the pair a hash key such as `globals:`
    /// rather than the start of a symbol.
    pub(super) label_colon: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self::with_start_line(source, 1)
    }

    /// A lexer over the core library metorex loads at startup. Every position
    /// it hands out says so, which is how a tracepoint knows to pass over the
    /// library's own statements the way Ruby passes over its C code.
    /// A lexer over a library metorex carries. Like the prelude, it is
    /// metorex's own source rather than the program's, so a setting the
    /// program was started with says nothing about its literals.
    pub fn for_embedded_library(source: &'a str) -> Self {
        let mut made = Self::with_start_line(source, 1);
        // A library's own magic comment decides for it, and the setting the
        // program was started with does not reach it.
        if !names_string_literal_setting(source) {
            made.frozen_literals = false;
            made.mutable_literals = false;
        }
        made
    }

    /// Say what encoding the source is written in, for code that arrives as a
    /// string rather than as a file: an eval takes the encoding of the string
    /// it was handed, unless the code names one of its own.
    pub fn with_source_encoding(mut self, named: Option<String>) -> Self {
        if let Some(named) = named {
            self.source_encoding = named;
        }
        self
    }

    pub fn for_prelude(source: &'a str) -> Self {
        let mut made = Self::with_start_line(source, 1);
        made.prelude = true;
        // The library metorex loads at startup is its own rather than the
        // program's, so a setting the program was started with says nothing
        // about it.
        made.frozen_literals = false;
        made.mutable_literals = false;
        made
    }

    /// Create a lexer whose first line is numbered `start_line`. Used by
    /// `eval`/`class_eval`/`module_eval` so `__LINE__` reflects the optional
    /// `lineno` argument (e.g. `class_eval("...", "file", 102)`).
    pub fn with_start_line(source: &'a str, start_line: usize) -> Self {
        // A file written with a byte order mark opens with one, and it names
        // the encoding rather than anything the program says.
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        let binary_source = names_binary_encoding(source);
        // A magic comment decides for the source it is written in; without
        // one the setting the program was started with decides.
        let (frozen_literals, mutable_literals) = if names_string_literal_setting(source) {
            let frozen = freezes_string_literals(source);
            (frozen, !frozen)
        } else {
            match literal_default() {
                LiteralDefault::Frozen => (true, false),
                LiteralDefault::Mutable => (false, true),
                LiteralDefault::Chilled => (false, false),
            }
        };
        Self {
            chars: source.chars().peekable(),
            prepend: Vec::new(),
            line: start_line,
            column: 1,
            offset: 0,
            prev_significant: None,
            prev_significant_end: 0,
            local_names: std::collections::HashSet::new(),
            alias_names_left: 0,
            def_parameters: DefParameters::Idle,
            closed_def_parameters: false,
            label_colon: false,
            restore_line: None,
            restore_offset: None,
            prelude: false,
            binary_source,
            frozen_literals,
            mutable_literals,
            source_encoding: named_source_encoding(source).unwrap_or_else(default_source_encoding),
        }
    }

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Token {
        // Save current state
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let saved_prev = self.prev_significant.clone();
        let saved_prev_end = self.prev_significant_end;
        let saved_restore_line = self.restore_line;
        let saved_restore_offset = self.restore_offset;
        let saved_label_colon = self.label_colon;

        // Get the next token
        let token = self.next_token();

        // Restore state
        self.chars = saved_chars;
        self.prepend = saved_prepend;
        self.line = saved_line;
        self.column = saved_column;
        self.offset = saved_offset;
        self.prev_significant = saved_prev;
        self.prev_significant_end = saved_prev_end;
        self.restore_line = saved_restore_line;
        self.restore_offset = saved_restore_offset;
        self.label_colon = saved_label_colon;

        token
    }

    /// Collect all tokens from the lexer
    pub fn tokenize(mut self) -> Vec<Token> {
        let _counted =
            crate::memory_category::attribute(crate::memory_category::Category::SyntaxTree);
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.kind == TokenKind::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    /// Get the next token from the source code, updating regex disambiguation state.
    pub fn next_token(&mut self) -> Token {
        let token = self.next_token_inner();
        match &token.kind {
            // Newlines reset expression context for slash-vs-regex purposes:
            // a `/` at the start of a fresh line should be parsed as a regex
            // literal, not division applied to whatever ended the prior line.
            TokenKind::Newline => {
                self.prev_significant = None;
                self.closed_def_parameters = false;
                if self.def_parameters == DefParameters::AfterDef {
                    self.def_parameters = DefParameters::Idle;
                }
            }
            // Comments and EOF leave the previous significant token alone.
            TokenKind::Comment(_) | TokenKind::EOF => {}
            other => {
                if let Some(TokenKind::Ident(name)) = &self.prev_significant
                    && binds_the_name_before_it(other)
                {
                    self.local_names.insert(name.clone());
                }
                self.alias_names_left = match other {
                    TokenKind::Alias => 2,
                    _ => self.alias_names_left.saturating_sub(1),
                };
                let adjacent = token.position.offset as usize == self.prev_significant_end;
                self.closed_def_parameters = false;
                self.def_parameters = match (self.def_parameters, other) {
                    (_, TokenKind::Def) => DefParameters::AfterDef,
                    (DefParameters::AfterDef, TokenKind::LParen) if adjacent => {
                        DefParameters::Open(1)
                    }
                    (
                        DefParameters::AfterDef,
                        TokenKind::LParen | TokenKind::Semicolon | TokenKind::Equal,
                    ) => DefParameters::Idle,
                    (DefParameters::Open(depth), TokenKind::LParen) => {
                        DefParameters::Open(depth + 1)
                    }
                    (DefParameters::Open(1), TokenKind::RParen) => {
                        self.closed_def_parameters = true;
                        DefParameters::Idle
                    }
                    (DefParameters::Open(depth), TokenKind::RParen) => {
                        DefParameters::Open(depth - 1)
                    }
                    (held, _) => held,
                };
                self.label_colon = matches!(other, TokenKind::Colon)
                    && matches!(self.prev_significant, Some(TokenKind::Ident(_)))
                    && token.position.offset as usize == self.prev_significant_end;
                self.prev_significant = Some(other.clone());
                self.prev_significant_end = self.offset;
            }
        }
        token
    }
}

/// Iterator implementation for Lexer
/// This allows using the lexer in for loops and with iterator methods
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::EOF {
            None
        } else {
            Some(token)
        }
    }
}

/// Whether a magic comment on one of the first two lines says the source is
/// written in bytes. Ruby reads such a file as bytes, so what a numeric
/// escape names is a byte rather than part of a character.
/// The tokens of source counted from line one, renumbered so the first line
/// is `start_line`, which Ruby lets an `eval` name as zero or below. Lines that
/// fall below one read as line zero.
pub fn tokens_from_line(tokens: &mut [Token], start_line: i64) {
    let shift = start_line - 1;
    if shift == 0 {
        return;
    }
    let moved = |line: i64| (line + shift).max(0);
    for token in tokens {
        token.position.line = moved(i64::from(token.position.line)) as u32;
        if let TokenKind::InterpolatedString(parts) | TokenKind::CommandString(parts) =
            &mut token.kind
        {
            for part in parts {
                if let InterpolationPart::Expression(_, line) = part
                    && *line > 0
                {
                    *line = moved(*line as i64) as usize;
                }
            }
        }
    }
}

/// Whether the source says its string literals are frozen, which the magic
/// comment `# frozen_string_literal: true` asks for.
pub(crate) fn freezes_string_literals(source: &str) -> bool {
    for line in source.lines().take(3) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        let Some(value) = frozen_string_literal_setting(trimmed) else {
            continue;
        };
        return value.starts_with("true");
    }
    false
}

/// The value a comment line sets `frozen_string_literal` to, the text after
/// the name and its colon. The name may be spelled with dashes for
/// underscores, and a mention of the name with no colon after it sets
/// nothing.
fn frozen_string_literal_setting(comment: &str) -> Option<String> {
    let lowered = comment.to_ascii_lowercase().replace('-', "_");
    let name = "frozen_string_literal";
    let mut from = 0;
    while let Some(found) = lowered[from..].find(name) {
        let after = &lowered[from + found + name.len()..];
        if let Some(value) = after.trim_start().strip_prefix(':') {
            return Some(value.trim_start().to_string());
        }
        from += found + name.len();
    }
    None
}

/// What a literal is in a source that says nothing about frozen string
/// literals: frozen, mutable, or handed back with notice that a later
/// release will freeze it.
#[derive(Clone, Copy, PartialEq)]
pub enum LiteralDefault {
    Frozen,
    Mutable,
    Chilled,
}

static LITERAL_DEFAULT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// The encoding a source is read as when it names none of its own, which is
/// what `-K` and `-U` say for the whole run.
static DEFAULT_SOURCE_ENCODING: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Say what encoding a source is written in when it names none of its own.
pub fn set_default_source_encoding(named: &str) {
    if let Ok(mut held) = DEFAULT_SOURCE_ENCODING.write() {
        *held = Some(named.to_string());
    }
}

/// The encoding `-K` named for the main script, when it named one.
pub fn command_line_source_encoding() -> Option<String> {
    DEFAULT_SOURCE_ENCODING
        .read()
        .ok()
        .and_then(|held| held.clone())
}

fn default_source_encoding() -> String {
    DEFAULT_SOURCE_ENCODING
        .read()
        .ok()
        .and_then(|held| held.clone())
        .unwrap_or_else(|| "UTF-8".to_string())
}

/// Say what a literal is in a source that names no setting of its own, which
/// is what `--enable-frozen-string-literal` and its opposite decide.
pub fn set_literal_default(setting: LiteralDefault) {
    let held = match setting {
        LiteralDefault::Chilled => 0,
        LiteralDefault::Frozen => 1,
        LiteralDefault::Mutable => 2,
    };
    LITERAL_DEFAULT.store(held, std::sync::atomic::Ordering::Relaxed);
}

fn literal_default() -> LiteralDefault {
    match LITERAL_DEFAULT.load(std::sync::atomic::Ordering::Relaxed) {
        1 => LiteralDefault::Frozen,
        2 => LiteralDefault::Mutable,
        _ => LiteralDefault::Chilled,
    }
}

/// The line of a `frozen_string_literal` magic comment written after code,
/// which names nothing. A line that only reads like one, inside a heredoc
/// or written as part of other words, is no magic comment.
pub fn frozen_string_literal_after_a_token(source: &str) -> Option<u32> {
    // Most files name the setting only in their first comments, so the
    // source is read as tokens only when a later line might.
    if !mentions_frozen_string_literal_after_code(source) {
        return None;
    }
    let mut a_token_came_first = false;
    for token in Lexer::new(source).tokenize() {
        match &token.kind {
            TokenKind::Newline | TokenKind::EOF => {}
            TokenKind::Comment(text) => {
                if a_token_came_first && is_frozen_string_literal_comment(text) {
                    return Some(token.position.line);
                }
            }
            _ => a_token_came_first = true,
        }
    }
    None
}

/// Whether some line after the first line of code holds a comment naming
/// `frozen_string_literal`.
fn mentions_frozen_string_literal_after_code(source: &str) -> bool {
    let mut a_token_came_first = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let Some(comment) = trimmed.strip_prefix('#') else {
            a_token_came_first = true;
            continue;
        };
        if a_token_came_first
            && comment
                .to_ascii_lowercase()
                .contains("frozen_string_literal")
        {
            return true;
        }
    }
    false
}

/// Whether a comment's text is a magic comment setting
/// `frozen_string_literal`, written alone, `frozen_string_literal: true`,
/// or among others between `-*-` marks.
fn is_frozen_string_literal_comment(text: &str) -> bool {
    let lowered = text
        .trim()
        .trim_start_matches('#')
        .trim()
        .to_ascii_lowercase();
    let setting = |written: &str| {
        let written = written.trim().replace('-', "_");
        written
            .strip_prefix("frozen_string_literal")
            .is_some_and(|rest| rest.trim_start().starts_with(':'))
    };
    match lowered
        .strip_prefix("-*-")
        .and_then(|inner| inner.strip_suffix("-*-"))
    {
        Some(inner) => inner.split(';').any(setting),
        None => setting(&lowered),
    }
}

/// Whether a magic comment says anything at all about frozen string
/// literals, whichever way it says it.
fn names_string_literal_setting(source: &str) -> bool {
    for line in source.lines().take(3) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        if frozen_string_literal_setting(trimmed).is_some() {
            return true;
        }
    }
    false
}

/// The source up to a line reading `__END__`, and where the data after that
/// line starts. Everything from there on is the program's data rather than
/// its code.
pub fn source_before_data_section(source: &str) -> (&str, Option<usize>) {
    let mut at = 0;
    while at <= source.len() {
        let line_end = source[at..]
            .find('\n')
            .map(|held| at + held)
            .unwrap_or(source.len());
        if &source[at..line_end] == "__END__" {
            let after = if line_end < source.len() {
                line_end + 1
            } else {
                line_end
            };
            return (&source[..at], Some(after));
        }
        if line_end >= source.len() {
            break;
        }
        at = line_end + 1;
    }
    (source, None)
}

/// What a comment names after `coding`, which Ruby reads only when a `:` or
/// an `=` separates the two. A comment merely holding the word, as in
/// "encoding-related", names no encoding at all.
fn encoding_after_coding(comment: &str) -> Option<String> {
    let lowered = comment.to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(found) = lowered[from..].find("coding") {
        let after = from + found + "coding".len();
        let rest = lowered[after..].trim_start();
        if let Some(named) = rest.strip_prefix(':').or_else(|| rest.strip_prefix('=')) {
            return Some(named.trim_start().to_string());
        }
        from = after;
    }
    None
}

/// The encoding the source's magic comment names, or None when it names none.
pub fn named_source_encoding(source: &str) -> Option<String> {
    let trimmed = magic_comment_line(source)?;
    let named = encoding_after_coding(trimmed)?;
    let spelled: String = named
        .chars()
        .take_while(|held| held.is_alphanumeric() || *held == '-' || *held == '_' || *held == '.')
        .collect();
    (!spelled.is_empty()).then_some(spelled)
}

fn names_binary_encoding(source: &str) -> bool {
    let Some(trimmed) = magic_comment_line(source) else {
        return false;
    };
    match encoding_after_coding(trimmed) {
        Some(named) => named.starts_with("binary") || named.starts_with("ascii-8bit"),
        None => false,
    }
}

/// The one line a magic comment can be written on: the first, or the second
/// when a shebang takes the first. A comment further down names nothing, so
/// an encoding comment after another magic comment is ignored.
fn magic_comment_line(source: &str) -> Option<&str> {
    let mut lines = source.lines();
    let first = lines.next()?.trim_start();
    let line = if first.starts_with("#!") {
        lines.next()?.trim_start()
    } else {
        first
    };
    line.starts_with('#').then_some(line)
}

/// Whether a token after a name marks the name as a local variable.
fn binds_the_name_before_it(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Equal
            | TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::PercentEqual
            | TokenKind::StarStarEqual
            | TokenKind::PipeEqual
            | TokenKind::AmpersandEqual
            | TokenKind::CaretEqual
            | TokenKind::ShovelEqual
            | TokenKind::RightShiftEqual
            | TokenKind::LogicalOrAssign
            | TokenKind::LogicalAndAssign
            | TokenKind::Comma
            | TokenKind::Pipe
            | TokenKind::RParen
            | TokenKind::In
    )
}
