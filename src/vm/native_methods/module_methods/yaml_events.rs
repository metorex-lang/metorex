// The parts of Psych that libyaml is under MRI: `Psych.__parse__` reads a
// stream into its events, and `Psych.__emitter__` writes events out as YAML,
// one Psych::Emitter at a time.

use super::*;
use crate::object::{OptionView, View};
use crate::yaml::{CollectionStyle, EventKind, Mark, ScalarStyle};

fn text_or_nil(text: &Option<String>) -> Object {
    match text {
        Some(held) => Object::string(held.clone()),
        None => Object::Nil,
    }
}

fn string_or_none(held: Option<&Object>) -> Option<String> {
    match held.view() {
        Some(View::String(text)) => Some(text.as_str().to_string()),
        _ => None,
    }
}

fn number_of(held: Option<&Object>) -> i64 {
    match held.view() {
        Some(View::Int(number)) => number,
        _ => 0,
    }
}

fn mark_fields(mark: Mark) -> [Object; 3] {
    [
        Object::Int(mark.index as i64),
        Object::Int(mark.line as i64),
        Object::Int(mark.column as i64),
    ]
}

/// An event as Psych's handler is called with it: its name, then the fields
/// that call takes.
fn event_fields(kind: &EventKind) -> Vec<Object> {
    match kind {
        EventKind::StreamStart => vec![Object::symbol("start_stream"), Object::Int(1)],
        EventKind::StreamEnd => vec![Object::symbol("end_stream")],
        EventKind::DocumentStart {
            version,
            tags,
            implicit,
        } => vec![
            Object::symbol("start_document"),
            Object::array(match version {
                Some((major, minor)) => {
                    vec![Object::Int(*major as i64), Object::Int(*minor as i64)]
                }
                None => Vec::new(),
            }),
            Object::array(
                tags.iter()
                    .map(|(handle, prefix)| {
                        Object::array(vec![
                            Object::string(handle.clone()),
                            Object::string(prefix.clone()),
                        ])
                    })
                    .collect(),
            ),
            Object::Bool(*implicit),
        ],
        EventKind::DocumentEnd { implicit } => {
            vec![Object::symbol("end_document"), Object::Bool(*implicit)]
        }
        EventKind::Alias { anchor } => {
            vec![Object::symbol("alias"), Object::string(anchor.clone())]
        }
        EventKind::Scalar {
            anchor,
            tag,
            value,
            plain_implicit,
            quoted_implicit,
            style,
        } => vec![
            Object::symbol("scalar"),
            Object::string(value.clone()),
            text_or_nil(anchor),
            text_or_nil(tag),
            Object::Bool(*plain_implicit),
            Object::Bool(*quoted_implicit),
            Object::Int(*style as i64),
        ],
        EventKind::SequenceStart {
            anchor,
            tag,
            implicit,
            style,
        } => vec![
            Object::symbol("start_sequence"),
            text_or_nil(anchor),
            text_or_nil(tag),
            Object::Bool(*implicit),
            Object::Int(*style as i64),
        ],
        EventKind::SequenceEnd => vec![Object::symbol("end_sequence")],
        EventKind::MappingStart {
            anchor,
            tag,
            implicit,
            style,
        } => vec![
            Object::symbol("start_mapping"),
            text_or_nil(anchor),
            text_or_nil(tag),
            Object::Bool(*implicit),
            Object::Int(*style as i64),
        ],
        EventKind::MappingEnd => vec![Object::symbol("end_mapping")],
    }
}

/// The event a Psych::Emitter call stands for, from its name and the
/// arguments the call was given.
fn event_from(fields: &[Object]) -> Option<EventKind> {
    let Some(View::Symbol(name)) = fields.first().view() else {
        return None;
    };
    let truthy = |at: usize| fields.get(at).is_some_and(crate::vm::utils::is_truthy);
    Some(match &*name.as_str() {
        "start_stream" => EventKind::StreamStart,
        "end_stream" => EventKind::StreamEnd,
        "start_document" => {
            let version = match fields.get(1).view() {
                Some(View::Array(parts)) if parts.borrow().len() >= 2 => {
                    let parts = parts.borrow();
                    Some((
                        number_of(parts.first()) as u32,
                        number_of(parts.get(1)) as u32,
                    ))
                }
                _ => None,
            };
            let tags = match fields.get(2).view() {
                Some(View::Array(pairs)) => pairs
                    .borrow()
                    .iter()
                    .filter_map(|pair| match pair.view() {
                        View::Array(parts) => {
                            let parts = parts.borrow();
                            Some((
                                string_or_none(parts.first()).unwrap_or_default(),
                                string_or_none(parts.get(1)).unwrap_or_default(),
                            ))
                        }
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            EventKind::DocumentStart {
                version,
                tags,
                implicit: truthy(3),
            }
        }
        "end_document" => EventKind::DocumentEnd {
            implicit: truthy(1),
        },
        "alias" => EventKind::Alias {
            anchor: string_or_none(fields.get(1)).unwrap_or_default(),
        },
        "scalar" => EventKind::Scalar {
            value: string_or_none(fields.get(1)).unwrap_or_default(),
            anchor: string_or_none(fields.get(2)),
            tag: string_or_none(fields.get(3)),
            plain_implicit: truthy(4),
            quoted_implicit: truthy(5),
            style: ScalarStyle::from_number(number_of(fields.get(6))),
        },
        "start_sequence" => EventKind::SequenceStart {
            anchor: string_or_none(fields.get(1)),
            tag: string_or_none(fields.get(2)),
            implicit: truthy(3),
            style: CollectionStyle::from_number(number_of(fields.get(4))),
        },
        "end_sequence" => EventKind::SequenceEnd,
        "start_mapping" => EventKind::MappingStart {
            anchor: string_or_none(fields.get(1)),
            tag: string_or_none(fields.get(2)),
            implicit: truthy(3),
            style: CollectionStyle::from_number(number_of(fields.get(4))),
        },
        "end_mapping" => EventKind::MappingEnd,
        _ => return None,
    })
}

impl VirtualMachine {
    /// `Psych.__parse__(bytes)`: every event the bytes read as, each the
    /// handler call's name and fields after its start and end marks and
    /// the scanner's place, and the error that stopped the reading or nil.
    pub(crate) fn yaml_parse(&mut self, arguments: &[Object]) -> Object {
        let bytes = match arguments.first().view() {
            Some(View::String(text)) => text.bytes().to_vec(),
            _ => Vec::new(),
        };
        let (events, error) = crate::yaml::parse(&bytes);
        let events = events
            .iter()
            .map(|event| {
                let mut fields = vec![
                    Object::Int(event.start.line as i64),
                    Object::Int(event.start.column as i64),
                    Object::Int(event.end.line as i64),
                    Object::Int(event.end.column as i64),
                ];
                fields.extend(mark_fields(event.reached));
                fields.extend(event_fields(&event.kind));
                Object::array(fields)
            })
            .collect();
        let error = match error {
            None => Object::Nil,
            Some(error) => Object::array(vec![
                Object::string(error.problem),
                text_or_nil(&error.context),
                Object::Int(error.context_mark.line as i64 + 1),
                Object::Int(error.context_mark.column as i64 + 1),
                Object::Int(error.offset as i64),
            ]),
        };
        Object::array(vec![Object::array(events), error])
    }

    /// `Psych.__emitter__(:new, line_width, indentation, canonical)` makes an
    /// emitter and answers its number, and `Psych.__emitter__(:emit, number,
    /// name, *fields)` writes one event and answers the text it finished.
    pub(crate) fn yaml_emitter(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let command = match arguments.first().view() {
            Some(View::Symbol(name)) => name.as_str().to_string(),
            _ => String::new(),
        };
        if command == "new" {
            let options = crate::yaml::EmitterOptions {
                line_width: number_of(arguments.get(1)),
                indentation: number_of(arguments.get(2)),
                canonical: arguments.get(3).is_some_and(crate::vm::utils::is_truthy),
            };
            self.next_yaml_emitter += 1;
            let number = self.next_yaml_emitter;
            self.yaml_emitters
                .insert(number, crate::yaml::Emitter::new(options));
            return Ok(Object::Int(number as i64));
        }
        let number = number_of(arguments.get(1)) as usize;
        let Some(event) = event_from(&arguments[2.min(arguments.len())..]) else {
            return Ok(Object::Nil);
        };
        let ends_the_stream = event == EventKind::StreamEnd;
        let Some(emitter) = self.yaml_emitters.get_mut(&number) else {
            return Err(crate::vm::errors::simple_exception(
                "RuntimeError",
                "expected nothing after STREAM-END",
                position,
            ));
        };
        let emitted = emitter.emit(event);
        let written = emitter.take_output();
        if ends_the_stream && emitted.is_ok() {
            self.yaml_emitters.remove(&number);
        }
        match emitted {
            Ok(()) => Ok(Object::string(written)),
            Err(problem) => Err(crate::vm::errors::simple_exception(
                "RuntimeError",
                &problem,
                position,
            )),
        }
    }
}
