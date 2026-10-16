//! What a body compiles to: its instructions and the side tables they name.

use crate::bytecode::instruction::Instruction;
use crate::lexer::Position;
use crate::object::Object;

/// A compiled body.
#[derive(Debug, Default)]
pub struct Unit {
    pub instructions: Vec<Instruction>,
    /// The source position of each instruction, which backtraces,
    /// `error_highlight`, and line events read.
    pub positions: Vec<Position>,
    /// The statement each instruction starts, if it starts one. A line event
    /// fires there and the interpreter checks for signals, thread switches,
    /// and collections, as it does before each statement it walks.
    pub statement_starts: Vec<Option<StatementStart>>,
    /// Where an error each instruction raises is pointed at, which
    /// `error_highlight` reads.
    pub error_spots: Vec<Option<ErrorSpot>>,
    pub constants: Vec<Object>,
    pub names: Vec<Name>,
    /// The unit's locals, numbered by their place here. `GetLocal` and
    /// `SetLocal` name one by that number.
    pub locals: Vec<Name>,
    pub call_sites: Vec<CallSite>,
    pub regexes: Vec<RegexLiteral>,
    pub hashes: Vec<HashShape>,
    pub interpolations: Vec<Vec<Piece>>,
    pub targets: Vec<Target>,
    pub multiple_assignments: Vec<MultipleAssignment>,
    /// The expressions `IsDefined` asks `defined?` about.
    pub defined: Vec<crate::ast::Expression>,
    /// The block and lambda literals `MakeBlock` makes Procs of.
    pub blocks: Vec<crate::ast::Expression>,
    /// The patterns `MatchPattern` matches, each with whether a value it does
    /// not match is refused, as `=>` refuses one.
    pub patterns: Vec<(crate::ast::MatchPattern, bool)>,
    pub handlers: Vec<Handler>,
    /// How many registers the unit's instructions name.
    pub register_count: usize,
}

/// The statement an instruction starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatementStart {
    /// The statement's place in the body, counting from 0.
    pub index: usize,
    /// The statement's own position, which its line event reports.
    pub position: Position,
}

/// The place in the source an error is pointed at, and the method or
/// variable name there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorSpot {
    pub start: Position,
    pub name: Option<String>,
}

/// A local, instance variable, constant, global, or literal text an
/// instruction names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub text: String,
}

impl Name {
    pub fn new(text: &str) -> Name {
        Name {
            text: text.to_string(),
        }
    }
}

/// How a call's receiver was written, which decides whether a private
/// method can be called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Receiver {
    /// The keyword `self`.
    SelfKeyword,
    /// The identifier `self`.
    SelfName,
    /// A value an operator assignment read, sent the method as `send` would
    /// send it.
    Sent,
    /// Any other expression.
    Other,
}

/// What a `Call` instruction calls.
#[derive(Debug, Clone)]
pub struct CallSite {
    pub name: String,
    /// How many registers after the receiver hold the arguments, when they
    /// are not gathered in an argument list.
    pub argument_count: usize,
    pub receiver: Receiver,
    /// Whether the arguments are gathered in the argument list
    /// `StartArguments` began, for a call with `*`, `**`, or `&` among them.
    pub arguments_in_list: bool,
    /// Whether a block written on the call is in the register after the
    /// arguments, or after the receiver when the arguments are in a list.
    pub has_block: bool,
    /// Whether the call was written with arguments, which a call without a
    /// receiver decides its route by.
    pub written_with_arguments: bool,
    /// Whether one of the arguments is `**value`.
    pub keyword_splat: bool,
}

impl CallSite {
    pub fn new(name: &str, argument_count: usize, receiver: Receiver) -> CallSite {
        CallSite {
            name: name.to_string(),
            argument_count,
            receiver,
            arguments_in_list: false,
            has_block: false,
            written_with_arguments: argument_count > 0,
            keyword_splat: false,
        }
    }

    /// The register holding the call's block, counted from the receiver's.
    pub fn block_offset(&self) -> usize {
        if self.arguments_in_list {
            1
        } else {
            1 + self.argument_count
        }
    }
}

/// The pattern and flags of a Regexp literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexLiteral {
    pub pattern: String,
    pub flags: String,
}

/// The registers a `NewHash` reads, from `first` on, one entry after the
/// other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashShape {
    pub first: usize,
    pub entries: Vec<HashEntry>,
}

/// One entry of a Hash literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashEntry {
    /// A key and its value, in two registers.
    Pair {
        key_position: Position,
        value_line: usize,
        /// Whether the key was written as a literal, the only keys a
        /// duplicate is warned about for.
        literal_key: bool,
    },
    /// `**value`, in one register.
    Spread { position: Position },
}

impl HashEntry {
    /// How many registers the entry reads.
    pub fn register_count(&self) -> usize {
        match self {
            HashEntry::Pair { .. } => 2,
            HashEntry::Spread { .. } => 1,
        }
    }
}

/// One piece of an interpolated String.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Text(String),
    /// A hole, whose value is in `register` and whose text, from `ToText`,
    /// is in the register after it.
    Value {
        register: usize,
        position: Position,
    },
}

/// An assignment target, with the registers holding what was evaluated
/// ahead of the value.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// A variable or a bare constant, which needs nothing evaluated first.
    Named(crate::ast::Expression),
    /// `receiver.reader = value`, or `receiver&.reader = value` when `safe`.
    Setter {
        receiver: usize,
        reader: String,
        safe: bool,
        names_self: bool,
        position: Position,
    },
    /// `receiver[subscripts] = value`, the subscripts in the `count`
    /// registers from `first`, or the elements of the Array in `first` when
    /// `spread`. `sends` is set for the forms that always write through a
    /// `[]=` call.
    Subscript {
        receiver: usize,
        first: usize,
        count: usize,
        spread: bool,
        sends: bool,
        position: Position,
    },
    /// `Namespace::Name = value`. `or_assigning` is set for `||=`, which
    /// reads a constant not defined yet as nil.
    ScopedConstant {
        namespace: usize,
        name: String,
        or_assigning: bool,
        position: Position,
    },
    /// `(a, b)` among the targets of a multiple assignment.
    Group(Vec<Target>, Position),
    /// `*rest` among the targets of a multiple assignment.
    Splat(Box<Target>),
}

/// The targets of a multiple assignment, and whether the right-hand side
/// was one value, which is spread across them, rather than several.
#[derive(Debug, Clone, PartialEq)]
pub struct MultipleAssignment {
    pub targets: Vec<Target>,
    pub lone_value: bool,
    pub position: Position,
}

/// Instructions from `start` up to `end` that raise an exception jump to
/// `target`, with the exception in `register`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handler {
    pub start: usize,
    pub end: usize,
    pub target: usize,
    pub register: usize,
}

/// Each table answers the index of the entry it adds. An index past 65,535
/// is named by an `Extend` before the instruction.
impl Unit {
    pub fn new() -> Unit {
        Unit::default()
    }

    /// Add an instruction, answering where it is.
    pub fn emit(&mut self, instruction: Instruction, position: Position) -> usize {
        self.instructions.push(instruction);
        self.positions.push(position);
        self.statement_starts.push(None);
        self.error_spots.push(None);
        self.instructions.len() - 1
    }

    /// Add an instruction that starts a statement, answering where it is.
    pub fn emit_statement(&mut self, instruction: Instruction, position: Position) -> usize {
        let at = self.emit(instruction, position);
        self.start_statement(at, position);
        at
    }

    /// Mark the instruction at `at` as the start of the next statement,
    /// written at `position`.
    pub fn start_statement(&mut self, at: usize, position: Position) {
        let index = self.statement_starts.iter().flatten().count();
        self.statement_starts[at] = Some(StatementStart { index, position });
    }

    /// Where the statement numbered `index` starts.
    pub fn statement_at(&self, index: usize) -> Option<usize> {
        self.statement_starts
            .iter()
            .position(|start| start.is_some_and(|start| start.index == index))
    }

    /// Give a register count of at least `count`.
    pub fn use_registers(&mut self, count: usize) {
        self.register_count = self.register_count.max(count);
    }

    pub fn constant(&mut self, value: Object) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn name(&mut self, name: Name) -> usize {
        self.names.push(name);
        self.names.len() - 1
    }

    /// The number of the local named `text`, adding it the first time.
    pub fn local(&mut self, text: &str) -> usize {
        match self.locals.iter().position(|held| held.text == text) {
            Some(at) => at,
            None => {
                self.locals.push(Name::new(text));
                self.locals.len() - 1
            }
        }
    }

    pub fn call_site(&mut self, site: CallSite) -> usize {
        self.call_sites.push(site);
        self.call_sites.len() - 1
    }

    pub fn regex(&mut self, regex: RegexLiteral) -> usize {
        self.regexes.push(regex);
        self.regexes.len() - 1
    }

    pub fn hash(&mut self, shape: HashShape) -> usize {
        self.hashes.push(shape);
        self.hashes.len() - 1
    }

    pub fn interpolation(&mut self, pieces: Vec<Piece>) -> usize {
        self.interpolations.push(pieces);
        self.interpolations.len() - 1
    }

    pub fn target(&mut self, target: Target) -> usize {
        self.targets.push(target);
        self.targets.len() - 1
    }

    pub fn multiple_assignment(&mut self, shape: MultipleAssignment) -> usize {
        self.multiple_assignments.push(shape);
        self.multiple_assignments.len() - 1
    }

    pub fn defined_expression(&mut self, expression: crate::ast::Expression) -> usize {
        self.defined.push(expression);
        self.defined.len() - 1
    }

    pub fn block(&mut self, literal: crate::ast::Expression) -> usize {
        self.blocks.push(literal);
        self.blocks.len() - 1
    }

    pub fn pattern(&mut self, pattern: crate::ast::MatchPattern, refuses: bool) -> usize {
        self.patterns.push((pattern, refuses));
        self.patterns.len() - 1
    }

    pub fn handler(&mut self, handler: Handler) {
        self.handlers.push(handler);
    }

    /// The handler covering the instruction at `at`, the innermost first.
    pub fn handler_for(&self, at: usize) -> Option<&Handler> {
        self.handlers
            .iter()
            .filter(|handler| handler.start <= at && at < handler.end)
            .min_by_key(|handler| handler.end - handler.start)
    }
}
