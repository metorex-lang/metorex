// Tokens read into events with libyaml's parser states, so a stream gives
// Psych the events MRI's parser gives it.

use super::scanner::{Scanner, Token, TokenKind};
use super::{CollectionStyle, Event, EventKind, Mark, ScalarStyle, YamlError};

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    StreamStart,
    ImplicitDocumentStart,
    DocumentStart,
    DocumentContent,
    DocumentEnd,
    BlockNode,
    BlockSequenceFirstEntry,
    BlockSequenceEntry,
    IndentlessSequenceEntry,
    BlockMappingFirstKey,
    BlockMappingKey,
    BlockMappingValue,
    FlowSequenceFirstEntry,
    FlowSequenceEntry,
    FlowSequenceEntryMappingKey,
    FlowSequenceEntryMappingValue,
    FlowSequenceEntryMappingEnd,
    FlowMappingFirstKey,
    FlowMappingKey,
    FlowMappingValue,
    FlowMappingEmptyValue,
    End,
}

struct Parser {
    scanner: Scanner,
    state: State,
    states: Vec<State>,
    marks: Vec<Mark>,
    tag_directives: Vec<(String, String)>,
}

/// Every event the text gives, and the error that stopped the reading when
/// one did, after the events read before it.
pub fn parse(bytes: &[u8]) -> (Vec<Event>, Option<YamlError>) {
    let (characters, trouble) = super::decode(bytes);
    let mut parser = Parser {
        scanner: Scanner::new(characters, trouble),
        state: State::StreamStart,
        states: Vec::new(),
        marks: Vec::new(),
        tag_directives: Vec::new(),
    };
    let mut events = Vec::new();
    while parser.state != State::End {
        match parser.next_event() {
            Ok(mut event) => {
                event.reached = parser.scanner.mark;
                events.push(event);
            }
            Err(error) => return (events, Some(error)),
        }
    }
    (events, None)
}

fn event(kind: EventKind, start: Mark, end: Mark) -> Event {
    Event {
        kind,
        start,
        end,
        reached: end,
    }
}

fn parser_error(problem: &str, problem_mark: Mark) -> YamlError {
    YamlError {
        context: None,
        context_mark: Mark::default(),
        problem: problem.to_string(),
        problem_mark,
        offset: 0,
    }
}

fn parser_error_in(context: &str, context_mark: Mark, problem: &str, problem_mark: Mark) -> YamlError {
    YamlError {
        context: Some(context.to_string()),
        context_mark,
        problem: problem.to_string(),
        problem_mark,
        offset: 0,
    }
}

impl Parser {
    fn peek(&mut self) -> Result<Token, YamlError> {
        self.scanner.peek_token().cloned()
    }

    fn skip(&mut self) {
        self.scanner.skip_token();
    }

    fn pop_state(&mut self) -> State {
        self.states.pop().unwrap_or(State::End)
    }

    fn next_event(&mut self) -> Result<Event, YamlError> {
        match self.state {
            State::StreamStart => self.stream_start(),
            State::ImplicitDocumentStart => self.document_start(true),
            State::DocumentStart => self.document_start(false),
            State::DocumentContent => self.document_content(),
            State::DocumentEnd => self.document_end(),
            State::BlockNode => self.node(true, false),
            State::BlockSequenceFirstEntry => self.block_sequence_entry(true),
            State::BlockSequenceEntry => self.block_sequence_entry(false),
            State::IndentlessSequenceEntry => self.indentless_sequence_entry(),
            State::BlockMappingFirstKey => self.block_mapping_key(true),
            State::BlockMappingKey => self.block_mapping_key(false),
            State::BlockMappingValue => self.block_mapping_value(),
            State::FlowSequenceFirstEntry => self.flow_sequence_entry(true),
            State::FlowSequenceEntry => self.flow_sequence_entry(false),
            State::FlowSequenceEntryMappingKey => self.flow_sequence_entry_mapping_key(),
            State::FlowSequenceEntryMappingValue => self.flow_sequence_entry_mapping_value(),
            State::FlowSequenceEntryMappingEnd => self.flow_sequence_entry_mapping_end(),
            State::FlowMappingFirstKey => self.flow_mapping_key(true),
            State::FlowMappingKey => self.flow_mapping_key(false),
            State::FlowMappingValue => self.flow_mapping_value(false),
            State::FlowMappingEmptyValue => self.flow_mapping_value(true),
            State::End => Err(parser_error("no more events", self.scanner.mark)),
        }
    }

    fn stream_start(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        if token.kind != TokenKind::StreamStart {
            return Err(parser_error("did not find expected <stream-start>", token.start));
        }
        self.state = State::ImplicitDocumentStart;
        self.skip();
        Ok(event(EventKind::StreamStart, token.start, token.end))
    }

    fn document_start(&mut self, implicit: bool) -> Result<Event, YamlError> {
        let mut token = self.peek()?;
        if !implicit {
            while token.kind == TokenKind::DocumentEnd {
                self.skip();
                token = self.peek()?;
            }
        }
        let opens_content = !matches!(
            token.kind,
            TokenKind::VersionDirective(..)
                | TokenKind::TagDirective(..)
                | TokenKind::DocumentStart
                | TokenKind::StreamEnd
        );
        if implicit && opens_content {
            self.process_directives()?;
            self.states.push(State::DocumentEnd);
            self.state = State::BlockNode;
            return Ok(event(
                EventKind::DocumentStart {
                    version: None,
                    tags: Vec::new(),
                    implicit: true,
                },
                token.start,
                token.start,
            ));
        }
        if token.kind != TokenKind::StreamEnd {
            let start = token.start;
            let (version, tags) = self.process_directives()?;
            let token = self.peek()?;
            if token.kind != TokenKind::DocumentStart {
                return Err(parser_error("did not find expected <document start>", token.start));
            }
            self.states.push(State::DocumentEnd);
            self.state = State::DocumentContent;
            self.skip();
            return Ok(event(
                EventKind::DocumentStart {
                    version,
                    tags,
                    implicit: false,
                },
                start,
                token.end,
            ));
        }
        self.state = State::End;
        self.skip();
        Ok(event(EventKind::StreamEnd, token.start, token.end))
    }

    /// The `%YAML` and `%TAG` directives before a document, which also put
    /// the default `!` and `!!` handles in force.
    #[allow(clippy::type_complexity)]
    fn process_directives(&mut self) -> Result<(Option<(u32, u32)>, Vec<(String, String)>), YamlError> {
        let mut version = None;
        let mut tags = Vec::new();
        loop {
            let token = self.peek()?;
            match token.kind {
                TokenKind::VersionDirective(major, minor) => {
                    if version.is_some() {
                        return Err(parser_error("found duplicate %YAML directive", token.start));
                    }
                    if major != 1 || (minor != 1 && minor != 2) {
                        return Err(parser_error("found incompatible YAML document", token.start));
                    }
                    version = Some((major, minor));
                }
                TokenKind::TagDirective(handle, prefix) => {
                    if self.tag_directives.iter().any(|(held, _)| *held == handle) {
                        return Err(parser_error("found duplicate %TAG directive", token.start));
                    }
                    self.tag_directives.push((handle.clone(), prefix.clone()));
                    tags.push((handle, prefix));
                }
                _ => break,
            }
            self.skip();
        }
        for (handle, prefix) in [("!", "!"), ("!!", "tag:yaml.org,2002:")] {
            if !self.tag_directives.iter().any(|(held, _)| held == handle) {
                self.tag_directives.push((handle.to_string(), prefix.to_string()));
            }
        }
        Ok((version, tags))
    }

    fn document_content(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        if matches!(
            token.kind,
            TokenKind::VersionDirective(..)
                | TokenKind::TagDirective(..)
                | TokenKind::DocumentStart
                | TokenKind::DocumentEnd
                | TokenKind::StreamEnd
        ) {
            self.state = self.pop_state();
            return Ok(empty_scalar(token.start));
        }
        self.node(true, false)
    }

    fn document_end(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        let start = token.start;
        let mut end = token.start;
        let mut implicit = true;
        if token.kind == TokenKind::DocumentEnd {
            end = token.end;
            self.skip();
            implicit = false;
        }
        self.tag_directives.clear();
        self.state = State::DocumentStart;
        Ok(event(EventKind::DocumentEnd { implicit }, start, end))
    }

    fn node(&mut self, block: bool, indentless_sequence: bool) -> Result<Event, YamlError> {
        let mut token = self.peek()?;
        if let TokenKind::Alias(anchor) = &token.kind {
            self.state = self.pop_state();
            self.skip();
            return Ok(event(
                EventKind::Alias {
                    anchor: anchor.clone(),
                },
                token.start,
                token.end,
            ));
        }
        let mut start = token.start;
        let mut end = token.start;
        let mut tag_mark = token.start;
        let mut anchor = None;
        let mut tag_parts = None;
        if let TokenKind::Anchor(value) = &token.kind {
            anchor = Some(value.clone());
            start = token.start;
            end = token.end;
            self.skip();
            token = self.peek()?;
            if let TokenKind::Tag(handle, suffix) = &token.kind {
                tag_parts = Some((handle.clone(), suffix.clone()));
                tag_mark = token.start;
                end = token.end;
                self.skip();
                token = self.peek()?;
            }
        } else if let TokenKind::Tag(handle, suffix) = &token.kind {
            tag_parts = Some((handle.clone(), suffix.clone()));
            start = token.start;
            tag_mark = token.start;
            end = token.end;
            self.skip();
            token = self.peek()?;
            if let TokenKind::Anchor(value) = &token.kind {
                anchor = Some(value.clone());
                end = token.end;
                self.skip();
                token = self.peek()?;
            }
        }
        let tag = match tag_parts {
            None => None,
            Some((handle, suffix)) if handle.is_empty() => Some(suffix),
            Some((handle, suffix)) => {
                match self.tag_directives.iter().find(|(held, _)| *held == handle) {
                    Some((_, prefix)) => Some(format!("{prefix}{suffix}")),
                    None => {
                        return Err(parser_error_in(
                            "while parsing a node",
                            start,
                            "found undefined tag handle",
                            tag_mark,
                        ));
                    }
                }
            }
        };
        let implicit = tag.as_deref().is_none_or(str::is_empty);
        if indentless_sequence && token.kind == TokenKind::BlockEntry {
            end = token.end;
            self.state = State::IndentlessSequenceEntry;
            return Ok(event(
                EventKind::SequenceStart {
                    anchor,
                    tag,
                    implicit,
                    style: CollectionStyle::Block,
                },
                start,
                end,
            ));
        }
        if let TokenKind::Scalar(value, style) = &token.kind {
            let mut plain_implicit = false;
            let mut quoted_implicit = false;
            end = token.end;
            if (*style == ScalarStyle::Plain && tag.is_none()) || tag.as_deref() == Some("!") {
                plain_implicit = true;
            } else if tag.is_none() {
                quoted_implicit = true;
            }
            self.state = self.pop_state();
            self.skip();
            return Ok(event(
                EventKind::Scalar {
                    anchor,
                    tag,
                    value: value.clone(),
                    plain_implicit,
                    quoted_implicit,
                    style: *style,
                },
                start,
                end,
            ));
        }
        let collection = match token.kind {
            TokenKind::FlowSequenceStart => Some((State::FlowSequenceFirstEntry, true, CollectionStyle::Flow)),
            TokenKind::FlowMappingStart => Some((State::FlowMappingFirstKey, false, CollectionStyle::Flow)),
            TokenKind::BlockSequenceStart if block => {
                Some((State::BlockSequenceFirstEntry, true, CollectionStyle::Block))
            }
            TokenKind::BlockMappingStart if block => {
                Some((State::BlockMappingFirstKey, false, CollectionStyle::Block))
            }
            _ => None,
        };
        if let Some((state, sequence, style)) = collection {
            end = token.end;
            self.state = state;
            let kind = if sequence {
                EventKind::SequenceStart {
                    anchor,
                    tag,
                    implicit,
                    style,
                }
            } else {
                EventKind::MappingStart {
                    anchor,
                    tag,
                    implicit,
                    style,
                }
            };
            return Ok(event(kind, start, end));
        }
        if anchor.is_some() || tag.is_some() {
            self.state = self.pop_state();
            return Ok(event(
                EventKind::Scalar {
                    anchor,
                    tag,
                    value: String::new(),
                    plain_implicit: implicit,
                    quoted_implicit: false,
                    style: ScalarStyle::Plain,
                },
                start,
                end,
            ));
        }
        Err(parser_error_in(
            if block {
                "while parsing a block node"
            } else {
                "while parsing a flow node"
            },
            start,
            "did not find expected node content",
            token.start,
        ))
    }

    fn block_sequence_entry(&mut self, first: bool) -> Result<Event, YamlError> {
        if first {
            let token = self.peek()?;
            self.marks.push(token.start);
            self.skip();
        }
        let token = self.peek()?;
        if token.kind == TokenKind::BlockEntry {
            let mark = token.end;
            self.skip();
            let token = self.peek()?;
            if !matches!(token.kind, TokenKind::BlockEntry | TokenKind::BlockEnd) {
                self.states.push(State::BlockSequenceEntry);
                return self.node(true, false);
            }
            self.state = State::BlockSequenceEntry;
            return Ok(empty_scalar(mark));
        }
        if token.kind == TokenKind::BlockEnd {
            self.state = self.pop_state();
            self.marks.pop();
            self.skip();
            return Ok(event(EventKind::SequenceEnd, token.start, token.end));
        }
        let opened = self.marks.pop().unwrap_or_default();
        Err(parser_error_in(
            "while parsing a block collection",
            opened,
            "did not find expected '-' indicator",
            token.start,
        ))
    }

    fn indentless_sequence_entry(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        if token.kind == TokenKind::BlockEntry {
            let mark = token.end;
            self.skip();
            let token = self.peek()?;
            if !matches!(
                token.kind,
                TokenKind::BlockEntry | TokenKind::Key | TokenKind::Value | TokenKind::BlockEnd
            ) {
                self.states.push(State::IndentlessSequenceEntry);
                return self.node(true, false);
            }
            self.state = State::IndentlessSequenceEntry;
            return Ok(empty_scalar(mark));
        }
        self.state = self.pop_state();
        Ok(event(EventKind::SequenceEnd, token.start, token.start))
    }

    fn block_mapping_key(&mut self, first: bool) -> Result<Event, YamlError> {
        if first {
            let token = self.peek()?;
            self.marks.push(token.start);
            self.skip();
        }
        let token = self.peek()?;
        if token.kind == TokenKind::Key {
            let mark = token.end;
            self.skip();
            let token = self.peek()?;
            if !matches!(token.kind, TokenKind::Key | TokenKind::Value | TokenKind::BlockEnd) {
                self.states.push(State::BlockMappingValue);
                return self.node(true, true);
            }
            self.state = State::BlockMappingValue;
            return Ok(empty_scalar(mark));
        }
        if token.kind == TokenKind::BlockEnd {
            self.state = self.pop_state();
            self.marks.pop();
            self.skip();
            return Ok(event(EventKind::MappingEnd, token.start, token.end));
        }
        let opened = self.marks.pop().unwrap_or_default();
        Err(parser_error_in(
            "while parsing a block mapping",
            opened,
            "did not find expected key",
            token.start,
        ))
    }

    fn block_mapping_value(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        if token.kind == TokenKind::Value {
            let mark = token.end;
            self.skip();
            let token = self.peek()?;
            if !matches!(token.kind, TokenKind::Key | TokenKind::Value | TokenKind::BlockEnd) {
                self.states.push(State::BlockMappingKey);
                return self.node(true, true);
            }
            self.state = State::BlockMappingKey;
            return Ok(empty_scalar(mark));
        }
        self.state = State::BlockMappingKey;
        Ok(empty_scalar(token.start))
    }

    fn flow_sequence_entry(&mut self, first: bool) -> Result<Event, YamlError> {
        if first {
            let token = self.peek()?;
            self.marks.push(token.start);
            self.skip();
        }
        let mut token = self.peek()?;
        if token.kind != TokenKind::FlowSequenceEnd {
            if !first {
                if token.kind == TokenKind::FlowEntry {
                    self.skip();
                    token = self.peek()?;
                } else {
                    let opened = self.marks.pop().unwrap_or_default();
                    return Err(parser_error_in(
                        "while parsing a flow sequence",
                        opened,
                        "did not find expected ',' or ']'",
                        token.start,
                    ));
                }
            }
            if token.kind == TokenKind::Key {
                self.state = State::FlowSequenceEntryMappingKey;
                self.skip();
                return Ok(event(
                    EventKind::MappingStart {
                        anchor: None,
                        tag: None,
                        implicit: true,
                        style: CollectionStyle::Flow,
                    },
                    token.start,
                    token.end,
                ));
            }
            if token.kind != TokenKind::FlowSequenceEnd {
                self.states.push(State::FlowSequenceEntry);
                return self.node(false, false);
            }
        }
        self.state = self.pop_state();
        self.marks.pop();
        self.skip();
        Ok(event(EventKind::SequenceEnd, token.start, token.end))
    }

    fn flow_sequence_entry_mapping_key(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        if !matches!(
            token.kind,
            TokenKind::Value | TokenKind::FlowEntry | TokenKind::FlowSequenceEnd
        ) {
            self.states.push(State::FlowSequenceEntryMappingValue);
            return self.node(false, false);
        }
        self.state = State::FlowSequenceEntryMappingValue;
        Ok(empty_scalar(token.start))
    }

    fn flow_sequence_entry_mapping_value(&mut self) -> Result<Event, YamlError> {
        let mut token = self.peek()?;
        if token.kind == TokenKind::Value {
            self.skip();
            token = self.peek()?;
            if !matches!(token.kind, TokenKind::FlowEntry | TokenKind::FlowSequenceEnd) {
                self.states.push(State::FlowSequenceEntryMappingEnd);
                return self.node(false, false);
            }
        }
        self.state = State::FlowSequenceEntryMappingEnd;
        Ok(empty_scalar(token.start))
    }

    fn flow_sequence_entry_mapping_end(&mut self) -> Result<Event, YamlError> {
        let token = self.peek()?;
        self.state = State::FlowSequenceEntry;
        Ok(event(EventKind::MappingEnd, token.start, token.start))
    }

    fn flow_mapping_key(&mut self, first: bool) -> Result<Event, YamlError> {
        if first {
            let token = self.peek()?;
            self.marks.push(token.start);
            self.skip();
        }
        let mut token = self.peek()?;
        if token.kind != TokenKind::FlowMappingEnd {
            if !first {
                if token.kind == TokenKind::FlowEntry {
                    self.skip();
                    token = self.peek()?;
                } else {
                    let opened = self.marks.pop().unwrap_or_default();
                    return Err(parser_error_in(
                        "while parsing a flow mapping",
                        opened,
                        "did not find expected ',' or '}'",
                        token.start,
                    ));
                }
            }
            if token.kind == TokenKind::Key {
                self.skip();
                token = self.peek()?;
                if !matches!(
                    token.kind,
                    TokenKind::Value | TokenKind::FlowEntry | TokenKind::FlowMappingEnd
                ) {
                    self.states.push(State::FlowMappingValue);
                    return self.node(false, false);
                }
                self.state = State::FlowMappingValue;
                return Ok(empty_scalar(token.start));
            }
            if token.kind != TokenKind::FlowMappingEnd {
                self.states.push(State::FlowMappingEmptyValue);
                return self.node(false, false);
            }
        }
        self.state = self.pop_state();
        self.marks.pop();
        self.skip();
        Ok(event(EventKind::MappingEnd, token.start, token.end))
    }

    fn flow_mapping_value(&mut self, empty: bool) -> Result<Event, YamlError> {
        let mut token = self.peek()?;
        if empty {
            self.state = State::FlowMappingKey;
            return Ok(empty_scalar(token.start));
        }
        if token.kind == TokenKind::Value {
            self.skip();
            token = self.peek()?;
            if !matches!(token.kind, TokenKind::FlowEntry | TokenKind::FlowMappingEnd) {
                self.states.push(State::FlowMappingKey);
                return self.node(false, false);
            }
        }
        self.state = State::FlowMappingKey;
        Ok(empty_scalar(token.start))
    }
}

fn empty_scalar(mark: Mark) -> Event {
    event(
        EventKind::Scalar {
            anchor: None,
            tag: None,
            value: String::new(),
            plain_implicit: true,
            quoted_implicit: false,
            style: ScalarStyle::Plain,
        },
        mark,
        mark,
    )
}
