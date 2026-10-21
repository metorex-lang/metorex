// Reading YAML text into tokens, the way libyaml's scanner does, so the
// events built from them are the ones Psych sees under MRI.

use std::collections::VecDeque;

use super::{Mark, ScalarStyle, YamlError};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TokenKind {
    StreamStart,
    StreamEnd,
    VersionDirective(u32, u32),
    TagDirective(String, String),
    DocumentStart,
    DocumentEnd,
    BlockSequenceStart,
    BlockMappingStart,
    BlockEnd,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    BlockEntry,
    FlowEntry,
    Key,
    Value,
    Alias(String),
    Anchor(String),
    Tag(String, String),
    Scalar(String, ScalarStyle),
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start: Mark,
    pub(crate) end: Mark,
}

/// A place a key may start, which a `:` after it turns into one.
#[derive(Clone, Copy, Default)]
struct SimpleKey {
    possible: bool,
    required: bool,
    token_number: usize,
    mark: Mark,
}

/// Why the text could not be read as characters, and the character it
/// stopped at.
pub(crate) struct ReaderTrouble {
    pub(crate) at: usize,
    pub(crate) offset: usize,
    pub(crate) problem: &'static str,
}

pub(crate) struct Scanner {
    input: Vec<char>,
    reader_trouble: Option<ReaderTrouble>,
    pointer: usize,
    pub(crate) mark: Mark,
    tokens: VecDeque<Token>,
    tokens_parsed: usize,
    token_available: bool,
    stream_start_produced: bool,
    stream_end_produced: bool,
    flow_level: usize,
    indent: i64,
    indents: Vec<i64>,
    simple_key_allowed: bool,
    simple_keys: Vec<SimpleKey>,
}

/// How many characters a key may run to before a `:` can no longer make it
/// one.
const SIMPLE_KEY_LENGTH: usize = 1024;

fn is_blank(held: char) -> bool {
    held == ' ' || held == '\t'
}

fn is_break(held: char) -> bool {
    matches!(held, '\r' | '\n' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

fn is_alpha(held: char) -> bool {
    held.is_ascii_alphanumeric() || held == '_' || held == '-'
}

fn hex_value(held: char) -> Option<u32> {
    held.to_digit(16)
}

impl Scanner {
    pub(crate) fn new(input: Vec<char>, reader_trouble: Option<ReaderTrouble>) -> Scanner {
        Scanner {
            input,
            reader_trouble,
            pointer: 0,
            mark: Mark::default(),
            tokens: VecDeque::new(),
            tokens_parsed: 0,
            token_available: false,
            stream_start_produced: false,
            stream_end_produced: false,
            flow_level: 0,
            indent: -1,
            indents: Vec::new(),
            simple_key_allowed: false,
            simple_keys: Vec::new(),
        }
    }

    fn at(&self, ahead: usize) -> char {
        self.input.get(self.pointer + ahead).copied().unwrap_or('\0')
    }

    fn at_end(&self) -> bool {
        self.pointer >= self.input.len()
    }

    fn is_blankz_at(&self, ahead: usize) -> bool {
        let held = self.at(ahead);
        is_blank(held) || is_break(held) || (held == '\0' && self.pointer + ahead >= self.input.len())
    }

    fn is_breakz_at(&self, ahead: usize) -> bool {
        let held = self.at(ahead);
        is_break(held) || (held == '\0' && self.pointer + ahead >= self.input.len())
    }

    fn skip(&mut self) {
        self.mark.index += 1;
        self.mark.column += 1;
        self.pointer += 1;
    }

    fn skip_line(&mut self) {
        if self.at(0) == '\r' && self.at(1) == '\n' {
            self.mark.index += 2;
            self.mark.column = 0;
            self.mark.line += 1;
            self.pointer += 2;
        } else if is_break(self.at(0)) {
            self.mark.index += 1;
            self.mark.column = 0;
            self.mark.line += 1;
            self.pointer += 1;
        }
    }

    fn read(&mut self, into: &mut String) {
        into.push(self.at(0));
        self.skip();
    }

    fn read_line(&mut self, into: &mut String) {
        let held = self.at(0);
        if held == '\r' && self.at(1) == '\n' {
            into.push('\n');
            self.pointer += 2;
            self.mark.index += 2;
        } else if held == '\r' || held == '\n' || held == '\u{85}' {
            into.push('\n');
            self.pointer += 1;
            self.mark.index += 1;
        } else if held == '\u{2028}' || held == '\u{2029}' {
            into.push(held);
            self.pointer += 1;
            self.mark.index += 1;
        } else {
            return;
        }
        self.mark.column = 0;
        self.mark.line += 1;
    }

    fn error(&self, context: &str, context_mark: Mark, problem: &str) -> YamlError {
        YamlError {
            context: Some(context.to_string()),
            context_mark,
            problem: problem.to_string(),
            problem_mark: self.mark,
            offset: 0,
        }
    }

    fn error_without_context(&self, problem: &str) -> YamlError {
        YamlError {
            context: None,
            context_mark: self.mark,
            problem: problem.to_string(),
            problem_mark: self.mark,
            offset: 0,
        }
    }

    /// The reader's complaint once the scanner has read up to where the text
    /// stops being characters it accepts.
    fn check_reader(&self) -> Result<(), YamlError> {
        if let Some(trouble) = &self.reader_trouble
            && self.pointer + 4 >= trouble.at
        {
            return Err(YamlError {
                context: None,
                context_mark: Mark::default(),
                problem: trouble.problem.to_string(),
                problem_mark: Mark::default(),
                offset: trouble.offset,
            });
        }
        Ok(())
    }

    pub(crate) fn peek_token(&mut self) -> Result<&Token, YamlError> {
        if !self.token_available {
            self.fetch_more_tokens()?;
        }
        self.tokens
            .front()
            .ok_or_else(|| self.error_without_context("no more tokens"))
    }

    pub(crate) fn skip_token(&mut self) {
        self.token_available = false;
        self.tokens_parsed += 1;
        if let Some(token) = self.tokens.pop_front()
            && token.kind == TokenKind::StreamEnd
        {
            self.stream_end_produced = true;
        }
    }

    fn fetch_more_tokens(&mut self) -> Result<(), YamlError> {
        loop {
            let need_more = if self.tokens.is_empty() {
                true
            } else {
                self.stale_simple_keys()?;
                self.simple_keys
                    .iter()
                    .any(|key| key.possible && key.token_number == self.tokens_parsed)
            };
            if !need_more {
                break;
            }
            if self.stream_end_produced {
                break;
            }
            self.fetch_next_token()?;
        }
        self.token_available = true;
        Ok(())
    }

    fn fetch_next_token(&mut self) -> Result<(), YamlError> {
        self.check_reader()?;
        if !self.stream_start_produced {
            return self.fetch_stream_start();
        }
        self.scan_to_next_token()?;
        self.check_reader()?;
        self.stale_simple_keys()?;
        self.unroll_indent(self.mark.column as i64);
        if self.at_end() {
            return self.fetch_stream_end();
        }
        let held = self.at(0);
        if self.mark.column == 0 && held == '%' {
            return self.fetch_directive();
        }
        if self.mark.column == 0
            && held == '-'
            && self.at(1) == '-'
            && self.at(2) == '-'
            && self.is_blankz_at(3)
        {
            return self.fetch_document_indicator(TokenKind::DocumentStart);
        }
        if self.mark.column == 0
            && held == '.'
            && self.at(1) == '.'
            && self.at(2) == '.'
            && self.is_blankz_at(3)
        {
            return self.fetch_document_indicator(TokenKind::DocumentEnd);
        }
        match held {
            '[' => return self.fetch_flow_collection_start(TokenKind::FlowSequenceStart),
            '{' => return self.fetch_flow_collection_start(TokenKind::FlowMappingStart),
            ']' => return self.fetch_flow_collection_end(TokenKind::FlowSequenceEnd),
            '}' => return self.fetch_flow_collection_end(TokenKind::FlowMappingEnd),
            ',' => return self.fetch_flow_entry(),
            _ => {}
        }
        if held == '-' && self.is_blankz_at(1) {
            return self.fetch_block_entry();
        }
        if held == '?' && (self.flow_level > 0 || self.is_blankz_at(1)) {
            return self.fetch_key();
        }
        if held == ':' && (self.flow_level > 0 || self.is_blankz_at(1)) {
            return self.fetch_value();
        }
        match held {
            '*' => return self.fetch_anchor(true),
            '&' => return self.fetch_anchor(false),
            '!' => return self.fetch_tag(),
            '|' if self.flow_level == 0 => return self.fetch_block_scalar(true),
            '>' if self.flow_level == 0 => return self.fetch_block_scalar(false),
            '\'' => return self.fetch_flow_scalar(true),
            '"' => return self.fetch_flow_scalar(false),
            _ => {}
        }
        let starts_plain = !(self.is_blankz_at(0)
            || matches!(
                held,
                '-' | '?'
                    | ':'
                    | ','
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '#'
                    | '&'
                    | '*'
                    | '!'
                    | '|'
                    | '>'
                    | '\''
                    | '"'
                    | '%'
                    | '@'
                    | '`'
            ))
            || (held == '-' && !is_blank(self.at(1)))
            || (self.flow_level == 0 && (held == '?' || held == ':') && !self.is_blankz_at(1));
        if starts_plain {
            return self.fetch_plain_scalar();
        }
        Err(self.error(
            "while scanning for the next token",
            self.mark,
            "found character that cannot start any token",
        ))
    }

    fn stale_simple_keys(&mut self) -> Result<(), YamlError> {
        let mark = self.mark;
        for index in 0..self.simple_keys.len() {
            let key = self.simple_keys[index];
            if key.possible
                && (key.mark.line < mark.line || key.mark.index + SIMPLE_KEY_LENGTH < mark.index)
            {
                if key.required {
                    return Err(self.error(
                        "while scanning a simple key",
                        key.mark,
                        "could not find expected ':'",
                    ));
                }
                self.simple_keys[index].possible = false;
            }
        }
        Ok(())
    }

    fn save_simple_key(&mut self) -> Result<(), YamlError> {
        let required = self.flow_level == 0 && self.indent == self.mark.column as i64;
        if self.simple_key_allowed {
            let key = SimpleKey {
                possible: true,
                required,
                token_number: self.tokens_parsed + self.tokens.len(),
                mark: self.mark,
            };
            self.remove_simple_key()?;
            if let Some(last) = self.simple_keys.last_mut() {
                *last = key;
            }
        }
        Ok(())
    }

    fn remove_simple_key(&mut self) -> Result<(), YamlError> {
        if let Some(key) = self.simple_keys.last().copied() {
            if key.possible && key.required {
                return Err(self.error(
                    "while scanning a simple key",
                    key.mark,
                    "could not find expected ':'",
                ));
            }
            if let Some(last) = self.simple_keys.last_mut() {
                last.possible = false;
            }
        }
        Ok(())
    }

    fn increase_flow_level(&mut self) {
        self.simple_keys.push(SimpleKey::default());
        self.flow_level += 1;
    }

    fn decrease_flow_level(&mut self) {
        if self.flow_level > 0 {
            self.flow_level -= 1;
            self.simple_keys.pop();
        }
    }

    fn roll_indent(&mut self, column: i64, number: Option<usize>, kind: TokenKind, mark: Mark) {
        if self.flow_level > 0 {
            return;
        }
        if self.indent < column {
            self.indents.push(self.indent);
            self.indent = column;
            let token = Token {
                kind,
                start: mark,
                end: mark,
            };
            match number {
                None => self.tokens.push_back(token),
                Some(number) => self.tokens.insert(number - self.tokens_parsed, token),
            }
        }
    }

    fn unroll_indent(&mut self, column: i64) {
        if self.flow_level > 0 {
            return;
        }
        while self.indent > column {
            self.tokens.push_back(Token {
                kind: TokenKind::BlockEnd,
                start: self.mark,
                end: self.mark,
            });
            self.indent = self.indents.pop().unwrap_or(-1);
        }
    }

    fn push_token(&mut self, kind: TokenKind, start: Mark, end: Mark) {
        self.tokens.push_back(Token { kind, start, end });
    }

    fn fetch_stream_start(&mut self) -> Result<(), YamlError> {
        self.indent = -1;
        self.simple_keys.push(SimpleKey::default());
        self.simple_key_allowed = true;
        self.stream_start_produced = true;
        self.push_token(TokenKind::StreamStart, self.mark, self.mark);
        Ok(())
    }

    fn fetch_stream_end(&mut self) -> Result<(), YamlError> {
        if self.mark.column != 0 {
            self.mark.column = 0;
            self.mark.line += 1;
        }
        self.unroll_indent(-1);
        self.remove_simple_key()?;
        self.simple_key_allowed = false;
        self.push_token(TokenKind::StreamEnd, self.mark, self.mark);
        Ok(())
    }

    fn fetch_directive(&mut self) -> Result<(), YamlError> {
        self.unroll_indent(-1);
        self.remove_simple_key()?;
        self.simple_key_allowed = false;
        let token = self.scan_directive()?;
        self.tokens.push_back(token);
        Ok(())
    }

    fn fetch_document_indicator(&mut self, kind: TokenKind) -> Result<(), YamlError> {
        self.unroll_indent(-1);
        self.remove_simple_key()?;
        self.simple_key_allowed = false;
        let start = self.mark;
        self.skip();
        self.skip();
        self.skip();
        self.push_token(kind, start, self.mark);
        Ok(())
    }

    fn fetch_flow_collection_start(&mut self, kind: TokenKind) -> Result<(), YamlError> {
        self.save_simple_key()?;
        self.increase_flow_level();
        self.simple_key_allowed = true;
        let start = self.mark;
        self.skip();
        self.push_token(kind, start, self.mark);
        Ok(())
    }

    fn fetch_flow_collection_end(&mut self, kind: TokenKind) -> Result<(), YamlError> {
        self.remove_simple_key()?;
        self.decrease_flow_level();
        self.simple_key_allowed = false;
        let start = self.mark;
        self.skip();
        self.push_token(kind, start, self.mark);
        Ok(())
    }

    fn fetch_flow_entry(&mut self) -> Result<(), YamlError> {
        self.remove_simple_key()?;
        self.simple_key_allowed = true;
        let start = self.mark;
        self.skip();
        self.push_token(TokenKind::FlowEntry, start, self.mark);
        Ok(())
    }

    fn fetch_block_entry(&mut self) -> Result<(), YamlError> {
        if self.flow_level == 0 {
            if !self.simple_key_allowed {
                return Err(
                    self.error_without_context("block sequence entries are not allowed in this context")
                );
            }
            self.roll_indent(
                self.mark.column as i64,
                None,
                TokenKind::BlockSequenceStart,
                self.mark,
            );
        }
        self.remove_simple_key()?;
        self.simple_key_allowed = true;
        let start = self.mark;
        self.skip();
        self.push_token(TokenKind::BlockEntry, start, self.mark);
        Ok(())
    }

    fn fetch_key(&mut self) -> Result<(), YamlError> {
        if self.flow_level == 0 {
            if !self.simple_key_allowed {
                return Err(self.error_without_context("mapping keys are not allowed in this context"));
            }
            self.roll_indent(
                self.mark.column as i64,
                None,
                TokenKind::BlockMappingStart,
                self.mark,
            );
        }
        self.remove_simple_key()?;
        self.simple_key_allowed = self.flow_level == 0;
        let start = self.mark;
        self.skip();
        self.push_token(TokenKind::Key, start, self.mark);
        Ok(())
    }

    fn fetch_value(&mut self) -> Result<(), YamlError> {
        let key = self.simple_keys.last().copied().unwrap_or_default();
        if key.possible {
            self.tokens.insert(
                key.token_number - self.tokens_parsed,
                Token {
                    kind: TokenKind::Key,
                    start: key.mark,
                    end: key.mark,
                },
            );
            self.roll_indent(
                key.mark.column as i64,
                Some(key.token_number),
                TokenKind::BlockMappingStart,
                key.mark,
            );
            if let Some(last) = self.simple_keys.last_mut() {
                last.possible = false;
            }
            self.simple_key_allowed = false;
        } else {
            if self.flow_level == 0 {
                if !self.simple_key_allowed {
                    return Err(
                        self.error_without_context("mapping values are not allowed in this context")
                    );
                }
                self.roll_indent(
                    self.mark.column as i64,
                    None,
                    TokenKind::BlockMappingStart,
                    self.mark,
                );
            }
            self.simple_key_allowed = self.flow_level == 0;
        }
        let start = self.mark;
        self.skip();
        self.push_token(TokenKind::Value, start, self.mark);
        Ok(())
    }

    fn fetch_anchor(&mut self, alias: bool) -> Result<(), YamlError> {
        self.save_simple_key()?;
        self.simple_key_allowed = false;
        let token = self.scan_anchor(alias)?;
        self.tokens.push_back(token);
        Ok(())
    }

    fn fetch_tag(&mut self) -> Result<(), YamlError> {
        self.save_simple_key()?;
        self.simple_key_allowed = false;
        let token = self.scan_tag()?;
        self.tokens.push_back(token);
        Ok(())
    }

    fn fetch_block_scalar(&mut self, literal: bool) -> Result<(), YamlError> {
        self.remove_simple_key()?;
        self.simple_key_allowed = true;
        let token = self.scan_block_scalar(literal)?;
        self.tokens.push_back(token);
        Ok(())
    }

    fn fetch_flow_scalar(&mut self, single: bool) -> Result<(), YamlError> {
        self.save_simple_key()?;
        self.simple_key_allowed = false;
        let token = self.scan_flow_scalar(single)?;
        self.tokens.push_back(token);
        Ok(())
    }

    fn fetch_plain_scalar(&mut self) -> Result<(), YamlError> {
        self.save_simple_key()?;
        self.simple_key_allowed = false;
        let token = self.scan_plain_scalar()?;
        self.tokens.push_back(token);
        Ok(())
    }

    fn scan_to_next_token(&mut self) -> Result<(), YamlError> {
        loop {
            if self.mark.column == 0 && self.at(0) == '\u{feff}' {
                self.skip();
            }
            while self.at(0) == ' '
                || ((self.flow_level > 0 || !self.simple_key_allowed) && self.at(0) == '\t')
            {
                self.skip();
            }
            if self.at(0) == '#' {
                while !self.is_breakz_at(0) {
                    self.skip();
                }
            }
            if is_break(self.at(0)) {
                self.skip_line();
                if self.flow_level == 0 {
                    self.simple_key_allowed = true;
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn scan_directive(&mut self) -> Result<Token, YamlError> {
        let start = self.mark;
        self.skip();
        let name = self.scan_directive_name(start)?;
        let kind = if name == "YAML" {
            let (major, minor) = self.scan_version_directive_value(start)?;
            TokenKind::VersionDirective(major, minor)
        } else if name == "TAG" {
            let (handle, prefix) = self.scan_tag_directive_value(start)?;
            TokenKind::TagDirective(handle, prefix)
        } else {
            return Err(self.error(
                "while scanning a directive",
                start,
                "found unknown directive name",
            ));
        };
        let end = self.mark;
        while is_blank(self.at(0)) {
            self.skip();
        }
        if self.at(0) == '#' {
            while !self.is_breakz_at(0) {
                self.skip();
            }
        }
        if !self.is_breakz_at(0) {
            return Err(self.error(
                "while scanning a directive",
                start,
                "did not find expected comment or line break",
            ));
        }
        if is_break(self.at(0)) {
            self.skip_line();
        }
        Ok(Token { kind, start, end })
    }

    fn scan_directive_name(&mut self, start: Mark) -> Result<String, YamlError> {
        let mut name = String::new();
        while is_alpha(self.at(0)) {
            self.read(&mut name);
        }
        if name.is_empty() {
            return Err(self.error(
                "while scanning a directive",
                start,
                "could not find expected directive name",
            ));
        }
        if !self.is_blankz_at(0) {
            return Err(self.error(
                "while scanning a directive",
                start,
                "found unexpected non-alphabetical character",
            ));
        }
        Ok(name)
    }

    fn scan_version_directive_value(&mut self, start: Mark) -> Result<(u32, u32), YamlError> {
        while is_blank(self.at(0)) {
            self.skip();
        }
        let major = self.scan_version_directive_number(start)?;
        if self.at(0) != '.' {
            return Err(self.error(
                "while scanning a %YAML directive",
                start,
                "did not find expected digit or '.' character",
            ));
        }
        self.skip();
        let minor = self.scan_version_directive_number(start)?;
        Ok((major, minor))
    }

    fn scan_version_directive_number(&mut self, start: Mark) -> Result<u32, YamlError> {
        let mut value: u32 = 0;
        let mut length = 0;
        while let Some(digit) = self.at(0).to_digit(10) {
            length += 1;
            if length > 9 {
                return Err(self.error(
                    "while scanning a %YAML directive",
                    start,
                    "found extremely long version number",
                ));
            }
            value = value * 10 + digit;
            self.skip();
        }
        if length == 0 {
            return Err(self.error(
                "while scanning a %YAML directive",
                start,
                "did not find expected version number",
            ));
        }
        Ok(value)
    }

    fn scan_tag_directive_value(&mut self, start: Mark) -> Result<(String, String), YamlError> {
        while is_blank(self.at(0)) {
            self.skip();
        }
        let handle = self.scan_tag_handle(true, start)?;
        if !is_blank(self.at(0)) {
            return Err(self.error(
                "while scanning a %TAG directive",
                start,
                "did not find expected whitespace",
            ));
        }
        while is_blank(self.at(0)) {
            self.skip();
        }
        let prefix = self.scan_tag_uri(true, true, None, start)?;
        if !self.is_blankz_at(0) {
            return Err(self.error(
                "while scanning a %TAG directive",
                start,
                "did not find expected whitespace or line break",
            ));
        }
        Ok((handle, prefix))
    }

    fn scan_anchor(&mut self, alias: bool) -> Result<Token, YamlError> {
        let start = self.mark;
        self.skip();
        let mut value = String::new();
        while is_alpha(self.at(0)) {
            self.read(&mut value);
        }
        let held = self.at(0);
        if value.is_empty()
            || !(self.is_blankz_at(0) || matches!(held, '?' | ':' | ',' | ']' | '}' | '%' | '@' | '`'))
        {
            return Err(self.error(
                if alias {
                    "while scanning an alias"
                } else {
                    "while scanning an anchor"
                },
                start,
                "did not find expected alphabetic or numeric character",
            ));
        }
        let kind = if alias {
            TokenKind::Alias(value)
        } else {
            TokenKind::Anchor(value)
        };
        Ok(Token {
            kind,
            start,
            end: self.mark,
        })
    }

    fn scan_tag(&mut self) -> Result<Token, YamlError> {
        let start = self.mark;
        let (handle, suffix);
        if self.at(1) == '<' {
            handle = String::new();
            self.skip();
            self.skip();
            suffix = self.scan_tag_uri(true, false, None, start)?;
            if self.at(0) != '>' {
                return Err(self.error("while scanning a tag", start, "did not find the expected '>'"));
            }
            self.skip();
        } else {
            let scanned = self.scan_tag_handle(false, start)?;
            if scanned.len() > 1 && scanned.starts_with('!') && scanned.ends_with('!') {
                handle = scanned;
                suffix = self.scan_tag_uri(false, false, None, start)?;
            } else {
                let rest = self.scan_tag_uri(false, false, Some(&scanned), start)?;
                if rest.is_empty() {
                    handle = String::new();
                    suffix = "!".to_string();
                } else {
                    handle = "!".to_string();
                    suffix = rest;
                }
            }
        }
        if !(self.is_blankz_at(0) || (self.flow_level > 0 && self.at(0) == ',')) {
            return Err(self.error(
                "while scanning a tag",
                start,
                "did not find expected whitespace or line break",
            ));
        }
        Ok(Token {
            kind: TokenKind::Tag(handle, suffix),
            start,
            end: self.mark,
        })
    }

    fn scan_tag_handle(&mut self, directive: bool, start: Mark) -> Result<String, YamlError> {
        let context = if directive {
            "while scanning a tag directive"
        } else {
            "while scanning a tag"
        };
        if self.at(0) != '!' {
            return Err(self.error(context, start, "did not find expected '!'"));
        }
        let mut handle = String::new();
        self.read(&mut handle);
        while is_alpha(self.at(0)) {
            self.read(&mut handle);
        }
        if self.at(0) == '!' {
            self.read(&mut handle);
        } else if directive && handle != "!" {
            return Err(self.error(
                "while parsing a tag directive",
                start,
                "did not find expected '!'",
            ));
        }
        Ok(handle)
    }

    fn scan_tag_uri(
        &mut self,
        uri_char: bool,
        directive: bool,
        head: Option<&str>,
        start: Mark,
    ) -> Result<String, YamlError> {
        let mut length = head.map_or(0, |head| head.chars().count());
        let mut text: String = head.map_or(String::new(), |head| head.chars().skip(1).collect());
        loop {
            let held = self.at(0);
            let allowed = is_alpha(held)
                || matches!(
                    held,
                    ';' | '/'
                        | '?'
                        | ':'
                        | '@'
                        | '&'
                        | '='
                        | '+'
                        | '$'
                        | '.'
                        | '%'
                        | '!'
                        | '~'
                        | '*'
                        | '\''
                        | '('
                        | ')'
                )
                || (uri_char && matches!(held, ',' | '[' | ']'));
            if !allowed {
                break;
            }
            if held == '%' {
                self.scan_uri_escapes(directive, start, &mut text)?;
            } else {
                self.read(&mut text);
            }
            length += 1;
        }
        if length == 0 {
            return Err(self.error(
                if directive {
                    "while parsing a %TAG directive"
                } else {
                    "while parsing a tag"
                },
                start,
                "did not find expected tag URI",
            ));
        }
        Ok(text)
    }

    fn scan_uri_escapes(
        &mut self,
        directive: bool,
        start: Mark,
        into: &mut String,
    ) -> Result<(), YamlError> {
        let context = if directive {
            "while parsing a %TAG directive"
        } else {
            "while parsing a tag"
        };
        let mut bytes = Vec::new();
        let mut width = 0;
        loop {
            let (Some(high), Some(low)) = (hex_value(self.at(1)), hex_value(self.at(2))) else {
                return Err(self.error(context, start, "did not find URI escaped octet"));
            };
            if self.at(0) != '%' {
                return Err(self.error(context, start, "did not find URI escaped octet"));
            }
            let octet = (high * 16 + low) as u8;
            if width == 0 {
                width = if octet & 0x80 == 0 {
                    1
                } else if octet & 0xE0 == 0xC0 {
                    2
                } else if octet & 0xF0 == 0xE0 {
                    3
                } else if octet & 0xF8 == 0xF0 {
                    4
                } else {
                    return Err(self.error(context, start, "found an incorrect leading UTF-8 octet"));
                };
            } else if octet & 0xC0 != 0x80 {
                return Err(self.error(context, start, "found an incorrect trailing UTF-8 octet"));
            }
            bytes.push(octet);
            self.skip();
            self.skip();
            self.skip();
            if bytes.len() == width {
                break;
            }
        }
        into.push_str(&String::from_utf8_lossy(&bytes));
        Ok(())
    }

    fn scan_block_scalar(&mut self, literal: bool) -> Result<Token, YamlError> {
        let start = self.mark;
        self.skip();
        let mut chomping = 0;
        let mut increment: i64 = 0;
        let context = "while scanning a block scalar";
        if self.at(0) == '+' || self.at(0) == '-' {
            chomping = if self.at(0) == '+' { 1 } else { -1 };
            self.skip();
            if let Some(digit) = self.at(0).to_digit(10) {
                if digit == 0 {
                    return Err(self.error(
                        context,
                        start,
                        "found an indentation indicator equal to 0",
                    ));
                }
                increment = digit as i64;
                self.skip();
            }
        } else if let Some(digit) = self.at(0).to_digit(10) {
            if digit == 0 {
                return Err(self.error(
                    context,
                    start,
                    "found an indentation indicator equal to 0",
                ));
            }
            increment = digit as i64;
            self.skip();
            if self.at(0) == '+' || self.at(0) == '-' {
                chomping = if self.at(0) == '+' { 1 } else { -1 };
                self.skip();
            }
        }
        while is_blank(self.at(0)) {
            self.skip();
        }
        if self.at(0) == '#' {
            while !self.is_breakz_at(0) {
                self.skip();
            }
        }
        if !self.is_breakz_at(0) {
            return Err(self.error(
                context,
                start,
                "did not find expected comment or line break",
            ));
        }
        if is_break(self.at(0)) {
            self.skip_line();
        }
        let mut end = self.mark;
        let mut indent: i64 = 0;
        if increment > 0 {
            indent = if self.indent >= 0 {
                self.indent + increment
            } else {
                increment
            };
        }
        let mut text = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();
        self.scan_block_scalar_breaks(&mut indent, &mut trailing_breaks, start, &mut end)?;
        let mut leading_blank = false;
        while self.mark.column as i64 == indent && !self.at_end() {
            let trailing_blank = is_blank(self.at(0));
            if !literal && leading_break.starts_with('\n') && !leading_blank && !trailing_blank {
                if trailing_breaks.is_empty() {
                    text.push(' ');
                }
                leading_break.clear();
            } else {
                text.push_str(&leading_break);
                leading_break.clear();
            }
            text.push_str(&trailing_breaks);
            trailing_breaks.clear();
            leading_blank = is_blank(self.at(0));
            while !self.is_breakz_at(0) {
                self.read(&mut text);
            }
            self.read_line(&mut leading_break);
            self.scan_block_scalar_breaks(&mut indent, &mut trailing_breaks, start, &mut end)?;
        }
        if chomping != -1 {
            text.push_str(&leading_break);
        }
        if chomping == 1 {
            text.push_str(&trailing_breaks);
        }
        Ok(Token {
            kind: TokenKind::Scalar(
                text,
                if literal {
                    ScalarStyle::Literal
                } else {
                    ScalarStyle::Folded
                },
            ),
            start,
            end,
        })
    }

    fn scan_block_scalar_breaks(
        &mut self,
        indent: &mut i64,
        breaks: &mut String,
        start: Mark,
        end: &mut Mark,
    ) -> Result<(), YamlError> {
        let mut max_indent: i64 = 0;
        *end = self.mark;
        loop {
            while (*indent == 0 || (self.mark.column as i64) < *indent) && self.at(0) == ' ' {
                self.skip();
            }
            if self.mark.column as i64 > max_indent {
                max_indent = self.mark.column as i64;
            }
            if (*indent == 0 || (self.mark.column as i64) < *indent) && self.at(0) == '\t' {
                return Err(self.error(
                    "while scanning a block scalar",
                    start,
                    "found a tab character where an indentation space is expected",
                ));
            }
            if !is_break(self.at(0)) {
                break;
            }
            self.read_line(breaks);
            *end = self.mark;
        }
        if *indent == 0 {
            *indent = max_indent;
            if *indent < self.indent + 1 {
                *indent = self.indent + 1;
            }
            if *indent < 1 {
                *indent = 1;
            }
        }
        Ok(())
    }

    fn at_document_indicator(&self) -> bool {
        self.mark.column == 0
            && ((self.at(0) == '-' && self.at(1) == '-' && self.at(2) == '-')
                || (self.at(0) == '.' && self.at(1) == '.' && self.at(2) == '.'))
            && self.is_blankz_at(3)
    }

    fn scan_flow_scalar(&mut self, single: bool) -> Result<Token, YamlError> {
        let start = self.mark;
        self.skip();
        let mut text = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();
        let mut whitespaces = String::new();
        loop {
            if self.at_document_indicator() {
                return Err(self.error(
                    "while scanning a quoted scalar",
                    start,
                    "found unexpected document indicator",
                ));
            }
            if self.at_end() {
                return Err(self.error(
                    "while scanning a quoted scalar",
                    start,
                    "found unexpected end of stream",
                ));
            }
            let mut leading_blanks = false;
            while !self.is_blankz_at(0) {
                let held = self.at(0);
                if single && held == '\'' && self.at(1) == '\'' {
                    text.push('\'');
                    self.skip();
                    self.skip();
                } else if held == (if single { '\'' } else { '"' }) {
                    break;
                } else if !single && held == '\\' && is_break(self.at(1)) {
                    self.skip();
                    self.skip_line();
                    leading_blanks = true;
                    break;
                } else if !single && held == '\\' {
                    let mut code_length = 0;
                    match self.at(1) {
                        '0' => text.push('\0'),
                        'a' => text.push('\u{7}'),
                        'b' => text.push('\u{8}'),
                        't' | '\t' => text.push('\t'),
                        'n' => text.push('\n'),
                        'v' => text.push('\u{b}'),
                        'f' => text.push('\u{c}'),
                        'r' => text.push('\r'),
                        'e' => text.push('\u{1b}'),
                        ' ' => text.push(' '),
                        '"' => text.push('"'),
                        '/' => text.push('/'),
                        '\'' => text.push('\''),
                        '\\' => text.push('\\'),
                        'N' => text.push('\u{85}'),
                        '_' => text.push('\u{a0}'),
                        'L' => text.push('\u{2028}'),
                        'P' => text.push('\u{2029}'),
                        'x' => code_length = 2,
                        'u' => code_length = 4,
                        'U' => code_length = 8,
                        _ => {
                            return Err(self.error(
                                "while parsing a quoted scalar",
                                start,
                                "found unknown escape character",
                            ));
                        }
                    }
                    self.skip();
                    self.skip();
                    if code_length > 0 {
                        let mut value: u32 = 0;
                        for place in 0..code_length {
                            let Some(digit) = hex_value(self.at(place)) else {
                                return Err(self.error(
                                    "while parsing a quoted scalar",
                                    start,
                                    "did not find expected hexdecimal number",
                                ));
                            };
                            value = (value << 4) + digit;
                        }
                        let Some(spelled) = char::from_u32(value) else {
                            return Err(self.error(
                                "while parsing a quoted scalar",
                                start,
                                "found invalid Unicode character escape code",
                            ));
                        };
                        text.push(spelled);
                        for _ in 0..code_length {
                            self.skip();
                        }
                    }
                } else {
                    self.read(&mut text);
                }
            }
            if self.at(0) == (if single { '\'' } else { '"' }) {
                break;
            }
            while is_blank(self.at(0)) || is_break(self.at(0)) {
                if is_blank(self.at(0)) {
                    if !leading_blanks {
                        self.read(&mut whitespaces);
                    } else {
                        self.skip();
                    }
                } else if !leading_blanks {
                    whitespaces.clear();
                    self.read_line(&mut leading_break);
                    leading_blanks = true;
                } else {
                    self.read_line(&mut trailing_breaks);
                }
            }
            if leading_blanks {
                if leading_break.starts_with('\n') {
                    if trailing_breaks.is_empty() {
                        text.push(' ');
                    } else {
                        text.push_str(&trailing_breaks);
                    }
                } else {
                    text.push_str(&leading_break);
                    text.push_str(&trailing_breaks);
                }
                leading_break.clear();
                trailing_breaks.clear();
            } else {
                text.push_str(&whitespaces);
                whitespaces.clear();
            }
        }
        self.skip();
        Ok(Token {
            kind: TokenKind::Scalar(
                text,
                if single {
                    ScalarStyle::SingleQuoted
                } else {
                    ScalarStyle::DoubleQuoted
                },
            ),
            start,
            end: self.mark,
        })
    }

    fn scan_plain_scalar(&mut self) -> Result<Token, YamlError> {
        let start = self.mark;
        let mut end = self.mark;
        let indent = self.indent + 1;
        let mut text = String::new();
        let mut leading_break = String::new();
        let mut trailing_breaks = String::new();
        let mut whitespaces = String::new();
        let mut leading_blanks = false;
        loop {
            if self.at_document_indicator() {
                break;
            }
            if self.at(0) == '#' {
                break;
            }
            while !self.is_blankz_at(0) {
                let held = self.at(0);
                if self.flow_level > 0
                    && held == ':'
                    && matches!(self.at(1), ',' | '?' | '[' | ']' | '{' | '}')
                {
                    return Err(self.error(
                        "while scanning a plain scalar",
                        start,
                        "found unexpected ':'",
                    ));
                }
                if (held == ':' && self.is_blankz_at(1))
                    || (self.flow_level > 0 && matches!(held, ',' | '[' | ']' | '{' | '}'))
                {
                    break;
                }
                if leading_blanks || !whitespaces.is_empty() {
                    if leading_blanks {
                        if leading_break.starts_with('\n') {
                            if trailing_breaks.is_empty() {
                                text.push(' ');
                            } else {
                                text.push_str(&trailing_breaks);
                            }
                        } else {
                            text.push_str(&leading_break);
                            text.push_str(&trailing_breaks);
                        }
                        leading_break.clear();
                        trailing_breaks.clear();
                        leading_blanks = false;
                    } else {
                        text.push_str(&whitespaces);
                        whitespaces.clear();
                    }
                }
                self.read(&mut text);
                end = self.mark;
            }
            if !(is_blank(self.at(0)) || is_break(self.at(0))) {
                break;
            }
            while is_blank(self.at(0)) || is_break(self.at(0)) {
                if is_blank(self.at(0)) {
                    if leading_blanks && (self.mark.column as i64) < indent && self.at(0) == '\t' {
                        return Err(self.error(
                            "while scanning a plain scalar",
                            start,
                            "found a tab character that violates indentation",
                        ));
                    }
                    if !leading_blanks {
                        self.read(&mut whitespaces);
                    } else {
                        self.skip();
                    }
                } else if !leading_blanks {
                    whitespaces.clear();
                    self.read_line(&mut leading_break);
                    leading_blanks = true;
                } else {
                    self.read_line(&mut trailing_breaks);
                }
            }
            if self.flow_level == 0 && (self.mark.column as i64) < indent {
                break;
            }
        }
        if leading_blanks {
            self.simple_key_allowed = true;
        }
        Ok(Token {
            kind: TokenKind::Scalar(text, ScalarStyle::Plain),
            start,
            end,
        })
    }
}
