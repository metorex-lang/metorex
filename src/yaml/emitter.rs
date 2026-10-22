// Events written back out as YAML text with libyaml's emitter states and
// its choice of how each scalar is written, so Psych.dump writes what it
// writes under MRI.

use std::collections::VecDeque;

use super::{CollectionStyle, EventKind, ScalarStyle};

/// What `Psych::Emitter.new` is handed: how wide a line may run, how far a
/// level is indented, and whether every node is written out in full.
#[derive(Clone, Copy, Debug)]
pub struct EmitterOptions {
    pub line_width: i64,
    pub indentation: i64,
    pub canonical: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    StreamStart,
    FirstDocumentStart,
    DocumentStart,
    DocumentContent,
    DocumentEnd,
    FlowSequenceFirstItem,
    FlowSequenceItem,
    FlowMappingFirstKey,
    FlowMappingKey,
    FlowMappingSimpleValue,
    FlowMappingValue,
    BlockSequenceFirstItem,
    BlockSequenceItem,
    BlockMappingFirstKey,
    BlockMappingKey,
    BlockMappingSimpleValue,
    BlockMappingValue,
    End,
}

#[derive(Default)]
struct AnchorData {
    anchor: Option<String>,
    alias: bool,
}

#[derive(Default)]
struct TagData {
    handle: Option<String>,
    suffix: Option<String>,
}

#[derive(Default)]
struct ScalarData {
    value: String,
    multiline: bool,
    flow_plain_allowed: bool,
    block_plain_allowed: bool,
    single_quoted_allowed: bool,
    block_allowed: bool,
    style: Option<ScalarStyle>,
}

pub struct Emitter {
    canonical: bool,
    best_indent: i64,
    best_width: i64,
    output: String,
    state: State,
    states: Vec<State>,
    events: VecDeque<EventKind>,
    indents: Vec<i64>,
    tag_directives: Vec<(String, String)>,
    indent: i64,
    flow_level: usize,
    root_context: bool,
    sequence_context: bool,
    mapping_context: bool,
    simple_key_context: bool,
    column: i64,
    whitespace: bool,
    indention: bool,
    open_ended: u8,
    anchor_data: AnchorData,
    tag_data: TagData,
    scalar_data: ScalarData,
}

type Emitted = Result<(), String>;

fn is_space(held: char) -> bool {
    held == ' '
}

fn is_break(held: char) -> bool {
    matches!(held, '\r' | '\n' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

fn is_blank(held: char) -> bool {
    held == ' ' || held == '\t'
}

fn is_alpha(held: char) -> bool {
    held.is_ascii_alphanumeric() || held == '_' || held == '-'
}

/// The characters libyaml's emitter writes as they are, outside double
/// quotes where it escapes them.
fn is_printable(held: char) -> bool {
    matches!(held, '\n' | '\u{20}'..='\u{7e}' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}')
        && held != '\u{feff}'
}

impl Emitter {
    pub fn new(options: EmitterOptions) -> Emitter {
        Emitter {
            canonical: options.canonical,
            best_indent: options.indentation,
            best_width: options.line_width,
            output: String::new(),
            state: State::StreamStart,
            states: Vec::new(),
            events: VecDeque::new(),
            indents: Vec::new(),
            tag_directives: Vec::new(),
            indent: -1,
            flow_level: 0,
            root_context: false,
            sequence_context: false,
            mapping_context: false,
            simple_key_context: false,
            column: 0,
            whitespace: true,
            indention: true,
            open_ended: 0,
            anchor_data: AnchorData::default(),
            tag_data: TagData::default(),
            scalar_data: ScalarData::default(),
        }
    }

    /// The text written since the last time it was taken.
    pub fn take_output(&mut self) -> String {
        std::mem::take(&mut self.output)
    }

    pub fn emit(&mut self, event: EventKind) -> Emitted {
        self.events.push_back(event);
        while !self.need_more_events() {
            let event = self.events.front().cloned().unwrap_or(EventKind::StreamEnd);
            self.analyze_event(&event)?;
            self.state_machine(&event)?;
            self.events.pop_front();
        }
        Ok(())
    }

    /// Whether the emitter waits for what follows the first event before it
    /// writes it: a collection is written differently when it is empty, and
    /// a key differently when it is short.
    fn need_more_events(&self) -> bool {
        let Some(head) = self.events.front() else {
            return true;
        };
        let accumulate = match head {
            EventKind::DocumentStart { .. } => 1,
            EventKind::SequenceStart { .. } => 2,
            EventKind::MappingStart { .. } => 3,
            _ => return false,
        };
        if self.events.len() > accumulate {
            return false;
        }
        let mut level: i64 = 0;
        for event in &self.events {
            match event {
                EventKind::StreamStart
                | EventKind::DocumentStart { .. }
                | EventKind::SequenceStart { .. }
                | EventKind::MappingStart { .. } => level += 1,
                EventKind::StreamEnd
                | EventKind::DocumentEnd { .. }
                | EventKind::SequenceEnd
                | EventKind::MappingEnd => level -= 1,
                _ => {}
            }
            if level == 0 {
                return false;
            }
        }
        true
    }

    fn analyze_event(&mut self, event: &EventKind) -> Emitted {
        self.anchor_data = AnchorData::default();
        self.tag_data = TagData::default();
        self.scalar_data = ScalarData::default();
        match event {
            EventKind::Alias { anchor } => self.analyze_anchor(anchor, true),
            EventKind::Scalar {
                anchor,
                tag,
                value,
                plain_implicit,
                quoted_implicit,
                ..
            } => {
                if let Some(anchor) = anchor {
                    self.analyze_anchor(anchor, false)?;
                }
                if let Some(tag) = tag
                    && (self.canonical || (!plain_implicit && !quoted_implicit))
                {
                    self.analyze_tag(tag)?;
                }
                self.analyze_scalar(value);
                Ok(())
            }
            EventKind::SequenceStart {
                anchor,
                tag,
                implicit,
                ..
            }
            | EventKind::MappingStart {
                anchor,
                tag,
                implicit,
                ..
            } => {
                if let Some(anchor) = anchor {
                    self.analyze_anchor(anchor, false)?;
                }
                if let Some(tag) = tag
                    && (self.canonical || !implicit)
                {
                    self.analyze_tag(tag)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn analyze_anchor(&mut self, anchor: &str, alias: bool) -> Emitted {
        let kind = if alias { "alias" } else { "anchor" };
        if anchor.is_empty() {
            return Err(format!("{kind} value must not be empty"));
        }
        if !anchor.chars().all(is_alpha) {
            return Err(format!(
                "{kind} value must contain alphanumerical characters only"
            ));
        }
        self.anchor_data = AnchorData {
            anchor: Some(anchor.to_string()),
            alias,
        };
        Ok(())
    }

    fn analyze_tag(&mut self, tag: &str) -> Emitted {
        if tag.is_empty() {
            return Err("tag value must not be empty".to_string());
        }
        for (handle, prefix) in &self.tag_directives {
            if prefix.len() < tag.len() && tag.starts_with(prefix.as_str()) {
                self.tag_data = TagData {
                    handle: Some(handle.clone()),
                    suffix: Some(tag[prefix.len()..].to_string()),
                };
                return Ok(());
            }
        }
        self.tag_data = TagData {
            handle: None,
            suffix: Some(tag.to_string()),
        };
        Ok(())
    }

    fn analyze_scalar(&mut self, value: &str) {
        self.scalar_data.value = value.to_string();
        if value.is_empty() {
            self.scalar_data.multiline = false;
            self.scalar_data.flow_plain_allowed = false;
            self.scalar_data.block_plain_allowed = true;
            self.scalar_data.single_quoted_allowed = true;
            self.scalar_data.block_allowed = false;
            return;
        }
        let characters: Vec<char> = value.chars().collect();
        let mut block_indicators = false;
        let mut flow_indicators = false;
        let mut line_breaks = false;
        let mut special_characters = false;
        let mut leading_space = false;
        let mut leading_break = false;
        let mut trailing_space = false;
        let mut trailing_break = false;
        let mut break_space = false;
        let mut space_break = false;
        let mut previous_space = false;
        let mut previous_break = false;
        if value.starts_with("---") || value.starts_with("...") {
            block_indicators = true;
            flow_indicators = true;
        }
        let blankz_at = |index: usize| {
            characters
                .get(index)
                .is_none_or(|held| is_blank(*held) || is_break(*held))
        };
        let mut preceded_by_whitespace = true;
        let mut followed_by_whitespace = blankz_at(1);
        let last = characters.len() - 1;
        for (index, held) in characters.iter().copied().enumerate() {
            if index == 0 {
                if matches!(
                    held,
                    '#' | ','
                        | '['
                        | ']'
                        | '{'
                        | '}'
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
                ) {
                    flow_indicators = true;
                    block_indicators = true;
                }
                if held == '?' || held == ':' {
                    flow_indicators = true;
                    if followed_by_whitespace {
                        block_indicators = true;
                    }
                }
                if held == '-' && followed_by_whitespace {
                    flow_indicators = true;
                    block_indicators = true;
                }
            } else {
                if matches!(held, ',' | '?' | '[' | ']' | '{' | '}') {
                    flow_indicators = true;
                }
                if held == ':' {
                    flow_indicators = true;
                    if followed_by_whitespace {
                        block_indicators = true;
                    }
                }
                if held == '#' && preceded_by_whitespace {
                    flow_indicators = true;
                    block_indicators = true;
                }
            }
            if !is_printable(held) {
                special_characters = true;
            }
            if is_break(held) {
                line_breaks = true;
            }
            if is_space(held) {
                if index == 0 {
                    leading_space = true;
                }
                if index == last {
                    trailing_space = true;
                }
                if previous_break {
                    break_space = true;
                }
                previous_space = true;
                previous_break = false;
            } else if is_break(held) {
                if index == 0 {
                    leading_break = true;
                }
                if index == last {
                    trailing_break = true;
                }
                if previous_space {
                    space_break = true;
                }
                previous_space = false;
                previous_break = true;
            } else {
                previous_space = false;
                previous_break = false;
            }
            preceded_by_whitespace = is_blank(held) || is_break(held);
            followed_by_whitespace = blankz_at(index + 2);
        }
        let data = &mut self.scalar_data;
        data.multiline = line_breaks;
        data.flow_plain_allowed = true;
        data.block_plain_allowed = true;
        data.single_quoted_allowed = true;
        data.block_allowed = true;
        if leading_space || leading_break || trailing_space || trailing_break {
            data.flow_plain_allowed = false;
            data.block_plain_allowed = false;
        }
        if trailing_space {
            data.block_allowed = false;
        }
        if break_space {
            data.flow_plain_allowed = false;
            data.block_plain_allowed = false;
            data.single_quoted_allowed = false;
        }
        if space_break || special_characters {
            data.flow_plain_allowed = false;
            data.block_plain_allowed = false;
            data.single_quoted_allowed = false;
            data.block_allowed = false;
        }
        if line_breaks {
            data.flow_plain_allowed = false;
            data.block_plain_allowed = false;
        }
        if flow_indicators {
            data.flow_plain_allowed = false;
        }
        if block_indicators {
            data.block_plain_allowed = false;
        }
    }

    fn state_machine(&mut self, event: &EventKind) -> Emitted {
        match self.state {
            State::StreamStart => self.emit_stream_start(event),
            State::FirstDocumentStart => self.emit_document_start(event, true),
            State::DocumentStart => self.emit_document_start(event, false),
            State::DocumentContent => self.emit_document_content(event),
            State::DocumentEnd => self.emit_document_end(event),
            State::FlowSequenceFirstItem => self.emit_flow_sequence_item(event, true),
            State::FlowSequenceItem => self.emit_flow_sequence_item(event, false),
            State::FlowMappingFirstKey => self.emit_flow_mapping_key(event, true),
            State::FlowMappingKey => self.emit_flow_mapping_key(event, false),
            State::FlowMappingSimpleValue => self.emit_flow_mapping_value(event, true),
            State::FlowMappingValue => self.emit_flow_mapping_value(event, false),
            State::BlockSequenceFirstItem => self.emit_block_sequence_item(event, true),
            State::BlockSequenceItem => self.emit_block_sequence_item(event, false),
            State::BlockMappingFirstKey => self.emit_block_mapping_key(event, true),
            State::BlockMappingKey => self.emit_block_mapping_key(event, false),
            State::BlockMappingSimpleValue => self.emit_block_mapping_value(event, true),
            State::BlockMappingValue => self.emit_block_mapping_value(event, false),
            State::End => Err("expected nothing after STREAM-END".to_string()),
        }
    }

    fn pop_state(&mut self) -> State {
        self.states.pop().unwrap_or(State::End)
    }

    fn emit_stream_start(&mut self, event: &EventKind) -> Emitted {
        if *event != EventKind::StreamStart {
            return Err("expected STREAM-START".to_string());
        }
        if self.best_indent < 2 || self.best_indent > 9 {
            self.best_indent = 2;
        }
        if self.best_width >= 0 && self.best_width <= self.best_indent * 2 {
            self.best_width = 80;
        }
        if self.best_width < 0 {
            self.best_width = i64::from(i32::MAX);
        }
        self.indent = -1;
        self.column = 0;
        self.whitespace = true;
        self.indention = true;
        self.state = State::FirstDocumentStart;
        Ok(())
    }

    fn emit_document_start(&mut self, event: &EventKind, first: bool) -> Emitted {
        match event {
            EventKind::DocumentStart {
                version,
                tags,
                implicit,
            } => {
                if let Some((major, minor)) = version
                    && (*major != 1 || (*minor != 1 && *minor != 2))
                {
                    return Err("incompatible %YAML directive".to_string());
                }
                for (handle, prefix) in tags {
                    analyze_tag_directive(handle, prefix)?;
                    if self.tag_directives.iter().any(|(held, _)| held == handle) {
                        return Err("duplicate %TAG directive".to_string());
                    }
                    self.tag_directives.push((handle.clone(), prefix.clone()));
                }
                for (handle, prefix) in [("!", "!"), ("!!", "tag:yaml.org,2002:")] {
                    if !self.tag_directives.iter().any(|(held, _)| held == handle) {
                        self.tag_directives
                            .push((handle.to_string(), prefix.to_string()));
                    }
                }
                let mut implicit = *implicit;
                if !first || self.canonical {
                    implicit = false;
                }
                if (version.is_some() || !tags.is_empty()) && self.open_ended > 0 {
                    self.write_indicator("...", true, false, false);
                    self.write_indent();
                }
                self.open_ended = 0;
                if let Some((_, minor)) = version {
                    implicit = false;
                    self.write_indicator("%YAML", true, false, false);
                    self.write_indicator(
                        if *minor == 1 { "1.1" } else { "1.2" },
                        true,
                        false,
                        false,
                    );
                    self.write_indent();
                }
                if !tags.is_empty() {
                    implicit = false;
                    for (handle, prefix) in tags {
                        self.write_indicator("%TAG", true, false, false);
                        self.write_tag_handle(handle);
                        self.write_tag_content(prefix, true);
                        self.write_indent();
                    }
                }
                if !implicit {
                    self.write_indent();
                    self.write_indicator("---", true, false, false);
                    if self.canonical {
                        self.write_indent();
                    }
                }
                self.state = State::DocumentContent;
                self.open_ended = 0;
                Ok(())
            }
            EventKind::StreamEnd => {
                if self.open_ended == 2 {
                    self.write_indicator("...", true, false, false);
                    self.open_ended = 0;
                    self.write_indent();
                }
                self.state = State::End;
                Ok(())
            }
            _ => Err("expected DOCUMENT-START or STREAM-END".to_string()),
        }
    }

    fn emit_document_content(&mut self, event: &EventKind) -> Emitted {
        self.states.push(State::DocumentEnd);
        self.emit_node(event, true, false, false, false)
    }

    fn emit_document_end(&mut self, event: &EventKind) -> Emitted {
        let EventKind::DocumentEnd { implicit } = event else {
            return Err("expected DOCUMENT-END".to_string());
        };
        self.write_indent();
        if !implicit {
            self.write_indicator("...", true, false, false);
            self.open_ended = 0;
            self.write_indent();
        } else if self.open_ended == 0 {
            self.open_ended = 1;
        }
        self.state = State::DocumentStart;
        self.tag_directives.clear();
        Ok(())
    }

    fn emit_flow_sequence_item(&mut self, event: &EventKind, first: bool) -> Emitted {
        if first {
            self.write_indicator("[", true, true, false);
            self.increase_indent(true, false);
            self.flow_level += 1;
        }
        if *event == EventKind::SequenceEnd {
            self.flow_level -= 1;
            self.indent = self.indents.pop().unwrap_or(-1);
            if self.canonical && !first {
                self.write_indicator(",", false, false, false);
                self.write_indent();
            }
            self.write_indicator("]", false, false, false);
            self.state = self.pop_state();
            return Ok(());
        }
        if !first {
            self.write_indicator(",", false, false, false);
        }
        if self.canonical || self.column > self.best_width {
            self.write_indent();
        }
        self.states.push(State::FlowSequenceItem);
        self.emit_node(event, false, true, false, false)
    }

    fn emit_flow_mapping_key(&mut self, event: &EventKind, first: bool) -> Emitted {
        if first {
            self.write_indicator("{", true, true, false);
            self.increase_indent(true, false);
            self.flow_level += 1;
        }
        if *event == EventKind::MappingEnd {
            self.flow_level -= 1;
            self.indent = self.indents.pop().unwrap_or(-1);
            if self.canonical && !first {
                self.write_indicator(",", false, false, false);
                self.write_indent();
            }
            self.write_indicator("}", false, false, false);
            self.state = self.pop_state();
            return Ok(());
        }
        if !first {
            self.write_indicator(",", false, false, false);
        }
        if self.canonical || self.column > self.best_width {
            self.write_indent();
        }
        if !self.canonical && self.check_simple_key() {
            self.states.push(State::FlowMappingSimpleValue);
            return self.emit_node(event, false, false, true, true);
        }
        self.write_indicator("?", true, false, false);
        self.states.push(State::FlowMappingValue);
        self.emit_node(event, false, false, true, false)
    }

    fn emit_flow_mapping_value(&mut self, event: &EventKind, simple: bool) -> Emitted {
        if simple {
            self.write_indicator(":", false, false, false);
        } else {
            if self.canonical || self.column > self.best_width {
                self.write_indent();
            }
            self.write_indicator(":", true, false, false);
        }
        self.states.push(State::FlowMappingKey);
        self.emit_node(event, false, false, true, false)
    }

    fn emit_block_sequence_item(&mut self, event: &EventKind, first: bool) -> Emitted {
        if first {
            let indentless = self.mapping_context && !self.indention;
            self.increase_indent(false, indentless);
        }
        if *event == EventKind::SequenceEnd {
            self.indent = self.indents.pop().unwrap_or(-1);
            self.state = self.pop_state();
            return Ok(());
        }
        self.write_indent();
        self.write_indicator("-", true, false, true);
        self.states.push(State::BlockSequenceItem);
        self.emit_node(event, false, true, false, false)
    }

    fn emit_block_mapping_key(&mut self, event: &EventKind, first: bool) -> Emitted {
        if first {
            self.increase_indent(false, false);
        }
        if *event == EventKind::MappingEnd {
            self.indent = self.indents.pop().unwrap_or(-1);
            self.state = self.pop_state();
            return Ok(());
        }
        self.write_indent();
        if self.check_simple_key() {
            self.states.push(State::BlockMappingSimpleValue);
            return self.emit_node(event, false, false, true, true);
        }
        self.write_indicator("?", true, false, true);
        self.states.push(State::BlockMappingValue);
        self.emit_node(event, false, false, true, false)
    }

    fn emit_block_mapping_value(&mut self, event: &EventKind, simple: bool) -> Emitted {
        if simple {
            self.write_indicator(":", false, false, false);
        } else {
            self.write_indent();
            self.write_indicator(":", true, false, true);
        }
        self.states.push(State::BlockMappingKey);
        self.emit_node(event, false, false, true, false)
    }

    fn emit_node(
        &mut self,
        event: &EventKind,
        root: bool,
        sequence: bool,
        mapping: bool,
        simple_key: bool,
    ) -> Emitted {
        self.root_context = root;
        self.sequence_context = sequence;
        self.mapping_context = mapping;
        self.simple_key_context = simple_key;
        match event {
            EventKind::Alias { .. } => {
                self.process_anchor();
                if self.simple_key_context {
                    self.put(' ');
                }
                self.state = self.pop_state();
                Ok(())
            }
            EventKind::Scalar { .. } => self.emit_scalar(event),
            EventKind::SequenceStart { style, .. } => {
                self.process_anchor();
                self.process_tag();
                self.state = if self.flow_level > 0
                    || self.canonical
                    || *style == CollectionStyle::Flow
                    || self.check_empty_sequence()
                {
                    State::FlowSequenceFirstItem
                } else {
                    State::BlockSequenceFirstItem
                };
                Ok(())
            }
            EventKind::MappingStart { style, .. } => {
                self.process_anchor();
                self.process_tag();
                self.state = if self.flow_level > 0
                    || self.canonical
                    || *style == CollectionStyle::Flow
                    || self.check_empty_mapping()
                {
                    State::FlowMappingFirstKey
                } else {
                    State::BlockMappingFirstKey
                };
                Ok(())
            }
            _ => Err("expected SCALAR, SEQUENCE-START, MAPPING-START, or ALIAS".to_string()),
        }
    }

    fn emit_scalar(&mut self, event: &EventKind) -> Emitted {
        self.select_scalar_style(event)?;
        self.process_anchor();
        self.process_tag();
        self.increase_indent(true, false);
        self.process_scalar();
        self.indent = self.indents.pop().unwrap_or(-1);
        self.state = self.pop_state();
        Ok(())
    }

    fn check_empty_sequence(&self) -> bool {
        self.events.len() >= 2
            && matches!(self.events[0], EventKind::SequenceStart { .. })
            && self.events[1] == EventKind::SequenceEnd
    }

    fn check_empty_mapping(&self) -> bool {
        self.events.len() >= 2
            && matches!(self.events[0], EventKind::MappingStart { .. })
            && self.events[1] == EventKind::MappingEnd
    }

    /// Whether the key at the head of the events fits on its line before
    /// the `:`, which a short single-line one does.
    fn check_simple_key(&self) -> bool {
        let Some(event) = self.events.front() else {
            return false;
        };
        let anchor_length = self.anchor_data.anchor.as_ref().map_or(0, String::len);
        let tag_length = self.tag_data.handle.as_ref().map_or(0, String::len)
            + self.tag_data.suffix.as_ref().map_or(0, String::len);
        let length = match event {
            EventKind::Alias { .. } => anchor_length,
            EventKind::Scalar { .. } => {
                if self.scalar_data.multiline {
                    return false;
                }
                anchor_length + tag_length + self.scalar_data.value.len()
            }
            EventKind::SequenceStart { .. } => {
                if !self.check_empty_sequence() {
                    return false;
                }
                anchor_length + tag_length
            }
            EventKind::MappingStart { .. } => {
                if !self.check_empty_mapping() {
                    return false;
                }
                anchor_length + tag_length
            }
            _ => return false,
        };
        length <= 128
    }

    fn select_scalar_style(&mut self, event: &EventKind) -> Emitted {
        let EventKind::Scalar {
            plain_implicit,
            quoted_implicit,
            style,
            ..
        } = event
        else {
            return Ok(());
        };
        let no_tag = self.tag_data.handle.is_none() && self.tag_data.suffix.is_none();
        if no_tag && !plain_implicit && !quoted_implicit {
            return Err("neither tag nor implicit flags are specified".to_string());
        }
        let mut style = *style;
        if style == ScalarStyle::Any {
            style = ScalarStyle::Plain;
        }
        if self.canonical {
            style = ScalarStyle::DoubleQuoted;
        }
        if self.simple_key_context && self.scalar_data.multiline {
            style = ScalarStyle::DoubleQuoted;
        }
        if style == ScalarStyle::Plain {
            if (self.flow_level > 0 && !self.scalar_data.flow_plain_allowed)
                || (self.flow_level == 0 && !self.scalar_data.block_plain_allowed)
            {
                style = ScalarStyle::SingleQuoted;
            }
            if self.scalar_data.value.is_empty() && (self.flow_level > 0 || self.simple_key_context)
            {
                style = ScalarStyle::SingleQuoted;
            }
            if no_tag && !plain_implicit {
                style = ScalarStyle::SingleQuoted;
            }
        }
        if style == ScalarStyle::SingleQuoted && !self.scalar_data.single_quoted_allowed {
            style = ScalarStyle::DoubleQuoted;
        }
        if (style == ScalarStyle::Literal || style == ScalarStyle::Folded)
            && (!self.scalar_data.block_allowed || self.flow_level > 0 || self.simple_key_context)
        {
            style = ScalarStyle::DoubleQuoted;
        }
        if no_tag && !quoted_implicit && style != ScalarStyle::Plain {
            self.tag_data.handle = Some("!".to_string());
        }
        self.scalar_data.style = Some(style);
        Ok(())
    }

    fn process_anchor(&mut self) {
        let Some(anchor) = self.anchor_data.anchor.clone() else {
            return;
        };
        let indicator = if self.anchor_data.alias { "*" } else { "&" };
        self.write_indicator(indicator, true, false, false);
        self.write_plain_text(&anchor);
    }

    fn process_tag(&mut self) {
        let handle = self.tag_data.handle.clone();
        let suffix = self.tag_data.suffix.clone();
        match (handle, suffix) {
            (None, None) => {}
            (Some(handle), suffix) => {
                self.write_tag_handle(&handle);
                if let Some(suffix) = suffix {
                    self.write_tag_content(&suffix, false);
                }
            }
            (None, Some(suffix)) => {
                self.write_indicator("!<", true, false, false);
                self.write_tag_content(&suffix, false);
                self.write_indicator(">", false, false, false);
            }
        }
    }

    fn process_scalar(&mut self) {
        let value = self.scalar_data.value.clone();
        let allow_breaks = !self.simple_key_context;
        match self.scalar_data.style.unwrap_or(ScalarStyle::Plain) {
            ScalarStyle::Plain | ScalarStyle::Any => self.write_plain(&value, allow_breaks),
            ScalarStyle::SingleQuoted => self.write_single_quoted(&value, allow_breaks),
            ScalarStyle::DoubleQuoted => self.write_double_quoted(&value, allow_breaks),
            ScalarStyle::Literal => self.write_literal(&value),
            ScalarStyle::Folded => self.write_folded(&value),
        }
    }

    fn increase_indent(&mut self, flow: bool, indentless: bool) {
        self.indents.push(self.indent);
        if self.indent < 0 {
            self.indent = if flow { self.best_indent } else { 0 };
        } else if !indentless {
            self.indent += self.best_indent;
        }
    }

    fn put(&mut self, held: char) {
        self.output.push(held);
        self.column += 1;
    }

    fn put_break(&mut self) {
        self.output.push('\n');
        self.column = 0;
    }

    /// A character of the value as it is, a line break written as one.
    fn write_character(&mut self, held: char) {
        if is_break(held) {
            if held == '\n' {
                self.put_break();
            } else {
                self.output.push(held);
                self.column = 0;
            }
        } else {
            self.put(held);
        }
    }

    fn write_plain_text(&mut self, text: &str) {
        for held in text.chars() {
            self.put(held);
        }
        self.whitespace = false;
        self.indention = false;
    }

    fn write_indent(&mut self) {
        let indent = self.indent.max(0);
        if !self.indention || self.column > indent || (self.column == indent && !self.whitespace) {
            self.put_break();
        }
        while self.column < indent {
            self.put(' ');
        }
        self.whitespace = true;
        self.indention = true;
    }

    fn write_indicator(
        &mut self,
        indicator: &str,
        need_whitespace: bool,
        is_whitespace: bool,
        is_indention: bool,
    ) {
        if need_whitespace && !self.whitespace {
            self.put(' ');
        }
        for held in indicator.chars() {
            self.put(held);
        }
        self.whitespace = is_whitespace;
        self.indention = self.indention && is_indention;
        self.open_ended = 0;
    }

    fn write_tag_handle(&mut self, handle: &str) {
        if !self.whitespace {
            self.put(' ');
        }
        self.write_plain_text(handle);
    }

    fn write_tag_content(&mut self, value: &str, need_whitespace: bool) {
        if need_whitespace && !self.whitespace {
            self.put(' ');
        }
        for held in value.chars() {
            if is_alpha(held)
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
                        | ','
                        | '_'
                        | '.'
                        | '~'
                        | '*'
                        | '\''
                        | '('
                        | ')'
                        | '['
                        | ']'
                )
            {
                self.put(held);
            } else {
                let mut bytes = [0; 4];
                for byte in held.encode_utf8(&mut bytes).bytes() {
                    self.put('%');
                    for digit in format!("{byte:02X}").chars() {
                        self.put(digit);
                    }
                }
            }
        }
        self.whitespace = false;
        self.indention = false;
    }

    fn write_plain(&mut self, value: &str, allow_breaks: bool) {
        if !self.whitespace && (!value.is_empty() || self.flow_level > 0) {
            self.put(' ');
        }
        let characters: Vec<char> = value.chars().collect();
        let mut spaces = false;
        let mut breaks = false;
        for (index, held) in characters.iter().copied().enumerate() {
            if is_space(held) {
                let next_is_space = characters
                    .get(index + 1)
                    .is_some_and(|next| is_space(*next));
                if allow_breaks && !spaces && self.column > self.best_width && !next_is_space {
                    self.write_indent();
                } else {
                    self.put(held);
                }
                spaces = true;
            } else if is_break(held) {
                if !breaks && held == '\n' {
                    self.put_break();
                }
                self.write_character(held);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                }
                self.put(held);
                self.indention = false;
                spaces = false;
                breaks = false;
            }
        }
        self.whitespace = false;
        self.indention = false;
        if self.root_context {
            self.open_ended = 1;
        }
    }

    fn write_single_quoted(&mut self, value: &str, allow_breaks: bool) {
        self.write_indicator("'", true, false, false);
        let characters: Vec<char> = value.chars().collect();
        let last = characters.len().saturating_sub(1);
        let mut spaces = false;
        let mut breaks = false;
        for (index, held) in characters.iter().copied().enumerate() {
            if is_space(held) {
                let next_is_space = characters
                    .get(index + 1)
                    .is_some_and(|next| is_space(*next));
                if allow_breaks
                    && !spaces
                    && self.column > self.best_width
                    && index != 0
                    && index != last
                    && !next_is_space
                {
                    self.write_indent();
                } else {
                    self.put(held);
                }
                spaces = true;
            } else if is_break(held) {
                if !breaks && held == '\n' {
                    self.put_break();
                }
                self.write_character(held);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                }
                if held == '\'' {
                    self.put('\'');
                }
                self.put(held);
                self.indention = false;
                spaces = false;
                breaks = false;
            }
        }
        if breaks {
            self.write_indent();
        }
        self.write_indicator("'", false, false, false);
        self.whitespace = false;
        self.indention = false;
    }

    fn write_double_quoted(&mut self, value: &str, allow_breaks: bool) {
        self.write_indicator("\"", true, false, false);
        let characters: Vec<char> = value.chars().collect();
        let last = characters.len().saturating_sub(1);
        let mut spaces = false;
        for (index, held) in characters.iter().copied().enumerate() {
            if !is_printable(held)
                || held == '\u{feff}'
                || is_break(held)
                || held == '"'
                || held == '\\'
            {
                self.put('\\');
                let short = match held {
                    '\0' => Some('0'),
                    '\u{7}' => Some('a'),
                    '\u{8}' => Some('b'),
                    '\t' => Some('t'),
                    '\n' => Some('n'),
                    '\u{b}' => Some('v'),
                    '\u{c}' => Some('f'),
                    '\r' => Some('r'),
                    '\u{1b}' => Some('e'),
                    '"' => Some('"'),
                    '\\' => Some('\\'),
                    '\u{85}' => Some('N'),
                    '\u{a0}' => Some('_'),
                    '\u{2028}' => Some('L'),
                    '\u{2029}' => Some('P'),
                    _ => None,
                };
                match short {
                    Some(letter) => self.put(letter),
                    None => {
                        let point = held as u32;
                        let written = if point <= 0xFF {
                            format!("x{point:02X}")
                        } else if point <= 0xFFFF {
                            format!("u{point:04X}")
                        } else {
                            format!("U{point:08X}")
                        };
                        for letter in written.chars() {
                            self.put(letter);
                        }
                    }
                }
                spaces = false;
            } else if is_space(held) {
                if allow_breaks
                    && !spaces
                    && self.column > self.best_width
                    && index != 0
                    && index != last
                {
                    self.write_indent();
                    if characters
                        .get(index + 1)
                        .is_some_and(|next| is_space(*next))
                    {
                        self.put('\\');
                    }
                } else {
                    self.put(held);
                }
                spaces = true;
            } else {
                self.put(held);
                spaces = false;
            }
        }
        self.write_indicator("\"", false, false, false);
        self.whitespace = false;
        self.indention = false;
    }

    fn write_block_scalar_hints(&mut self, value: &str) {
        let first = value.chars().next();
        if first.is_some_and(|held| is_space(held) || is_break(held)) {
            let hint = self.best_indent.to_string();
            self.write_indicator(&hint, false, false, false);
        }
        self.open_ended = 0;
        let mut from_end = value.chars().rev();
        let chomp_hint = match from_end.next() {
            None => Some("-"),
            Some(last) if !is_break(last) => Some("-"),
            Some(_) => match from_end.next() {
                None => {
                    self.open_ended = 2;
                    Some("+")
                }
                Some(before) if is_break(before) => {
                    self.open_ended = 2;
                    Some("+")
                }
                Some(_) => None,
            },
        };
        if let Some(hint) = chomp_hint {
            self.write_indicator(hint, false, false, false);
        }
    }

    fn write_literal(&mut self, value: &str) {
        self.write_indicator("|", true, false, false);
        self.write_block_scalar_hints(value);
        self.put_break();
        self.indention = true;
        self.whitespace = true;
        let mut breaks = true;
        for held in value.chars() {
            if is_break(held) {
                self.write_character(held);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                }
                self.put(held);
                self.indention = false;
                breaks = false;
            }
        }
    }

    fn write_folded(&mut self, value: &str) {
        self.write_indicator(">", true, false, false);
        self.write_block_scalar_hints(value);
        self.put_break();
        self.indention = true;
        self.whitespace = true;
        let characters: Vec<char> = value.chars().collect();
        let mut breaks = true;
        let mut leading_spaces = true;
        for (index, held) in characters.iter().copied().enumerate() {
            if is_break(held) {
                if !breaks && !leading_spaces && held == '\n' {
                    let mut ahead = index;
                    while characters.get(ahead).is_some_and(|next| is_break(*next)) {
                        ahead += 1;
                    }
                    let blankz = characters
                        .get(ahead)
                        .is_none_or(|next| is_blank(*next) || is_break(*next));
                    if !blankz {
                        self.put_break();
                    }
                }
                self.write_character(held);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                    leading_spaces = is_blank(held);
                }
                let next_is_space = characters
                    .get(index + 1)
                    .is_some_and(|next| is_space(*next));
                if !breaks && is_space(held) && !next_is_space && self.column > self.best_width {
                    self.write_indent();
                } else {
                    self.put(held);
                }
                self.indention = false;
                breaks = false;
            }
        }
    }
}

fn analyze_tag_directive(handle: &str, prefix: &str) -> Emitted {
    if handle.is_empty() {
        return Err("tag handle must not be empty".to_string());
    }
    if !handle.starts_with('!') {
        return Err("tag handle must start with '!'".to_string());
    }
    if !handle.ends_with('!') {
        return Err("tag handle must end with '!'".to_string());
    }
    let inner = &handle[1..handle.len().saturating_sub(1).max(1)];
    if !inner.chars().all(is_alpha) {
        return Err("tag handle must contain alphanumerical characters only".to_string());
    }
    if prefix.is_empty() {
        return Err("tag prefix must not be empty".to_string());
    }
    Ok(())
}
