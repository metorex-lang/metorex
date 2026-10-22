//! Call frame tracking for the Metorex virtual machine.
//!
//! This module provides call frame information used for debugging and stack traces.

use crate::scope::Scope;
use std::cell::RefCell;
use std::rc::Rc;

/// What kind of code a frame is running, which is what `__callee__` and
/// `__method__` walk the stack to find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameKind {
    /// A method activation. `callee` is the name the method was called by,
    /// which differs from `defined` when the method was reached by an alias.
    Method { callee: Rc<str>, defined: Rc<str> },
    /// A block body, which reports the method that encloses it.
    Block,
    /// A class body or a loaded file, where no method is running at all.
    Boundary,
}

/// Where a frame was entered from.
#[derive(Debug, Clone)]
enum FrameLocation {
    Written(String),
    At { line: usize, column: usize },
}

/// Call frame information stored on the VM call stack for debugging.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Human-readable frame identifier (method/function name). Shared, so
    /// a method's frames reuse the one name the method caches.
    name: Rc<str>,
    /// Optional source location ("file:line") to aid debugging.
    location: Option<FrameLocation>,
    /// The file the call site sits in, when known. `location` carries only a
    /// line and column, so this is what names the file for a backtrace.
    source_file: Option<Rc<str>>,
    /// What the frame is running.
    kind: FrameKind,
    /// How many blocks deep the frame sits inside the scope it names. Zero
    /// where the frame runs the scope itself rather than a block written in
    /// it, which is what tells `foo` from `block in foo`.
    block_depth: u32,
    /// The scope the block this frame runs was written in, for a block frame.
    written_in: Option<Rc<str>>,
    /// The class path of the module the running method was defined in, for
    /// a method frame or a block written in one. None for a singleton
    /// method.
    owner_path: Option<Rc<str>>,
    /// The class or module the method a block frame was written in was
    /// defined on.
    owner_class: Option<Rc<crate::class::Class>>,
    /// The scope that was current when the frame was pushed.
    entering_scope: Option<Rc<RefCell<Scope>>>,
    /// What a backtrace names the frame, where that is not its name: a
    /// method `define_method` made from a block is named as the block.
    label: Option<Rc<str>>,
}

impl CallFrame {
    /// Create a new call frame description for a block body.
    pub fn new(name: impl Into<Rc<str>>, location: Option<String>) -> Self {
        Self {
            name: name.into(),
            location: location.map(FrameLocation::Written),
            source_file: None,
            kind: FrameKind::Block,
            block_depth: 1,
            written_in: None,
            owner_path: None,
            owner_class: None,
            entering_scope: None,
            label: None,
        }
    }

    /// Create a frame for a method activation.
    pub fn method(
        name: impl Into<Rc<str>>,
        location: Option<String>,
        callee: impl Into<Rc<str>>,
        defined: impl Into<Rc<str>>,
    ) -> Self {
        Self {
            name: name.into(),
            location: location.map(FrameLocation::Written),
            source_file: None,
            kind: FrameKind::Method {
                callee: callee.into(),
                defined: defined.into(),
            },
            block_depth: 0,
            written_in: None,
            owner_path: None,
            owner_class: None,
            entering_scope: None,
            label: None,
        }
    }

    /// Create a frame for a class body or a loaded file.
    pub fn boundary(name: impl Into<Rc<str>>) -> Self {
        Self {
            name: name.into(),
            location: None,
            source_file: None,
            kind: FrameKind::Boundary,
            block_depth: 0,
            written_in: None,
            owner_path: None,
            owner_class: None,
            entering_scope: None,
            label: None,
        }
    }

    /// Name the frame in backtraces by `label` when there is one.
    pub fn labeled(mut self, label: Option<Rc<str>>) -> Self {
        self.label = label;
        self
    }

    /// What a backtrace names the frame, where that is not its name.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// What this frame is running.
    pub fn kind(&self) -> &FrameKind {
        &self.kind
    }

    /// Return the frame name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the optional source location.
    pub fn location(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.location.as_ref().map(|location| match location {
            FrameLocation::Written(text) => std::borrow::Cow::Borrowed(text.as_str()),
            FrameLocation::At { line, column } => {
                std::borrow::Cow::Owned(format!("{line}:{column}"))
            }
        })
    }

    /// Record the file the call site sits in.
    /// Record where the frame was entered from, which is what pairs a
    /// backtrace entry with the frame that made the call.
    pub fn with_location(mut self, location: Option<String>) -> Self {
        self.location = location.map(FrameLocation::Written);
        self
    }

    /// Record that the frame was entered from `line` and `column`, which
    /// `location` writes out as `line:column` when it is read.
    pub fn entered_at(mut self, line: usize, column: usize) -> Self {
        self.location = Some(FrameLocation::At { line, column });
        self
    }

    pub fn with_source_file(mut self, source_file: Option<impl Into<Rc<str>>>) -> Self {
        self.source_file = source_file.map(Into::into);
        self
    }

    /// The file the call site sits in, when it was recorded.
    pub fn source_file(&self) -> Option<&str> {
        self.source_file.as_deref()
    }

    /// Say the frame runs a block written `depth` blocks inside the scope it
    /// names.
    pub fn nested_in_a_block(mut self, depth: u32) -> Self {
        self.block_depth = depth;
        self
    }

    /// How many blocks deep the frame sits inside the scope it names.
    pub fn block_depth(&self) -> u32 {
        self.block_depth
    }

    /// Say which scope the block this frame runs was written in.
    pub fn written_in_scope(mut self, scope: Option<impl Into<Rc<str>>>) -> Self {
        self.written_in = scope.map(Into::into);
        self
    }

    /// The scope the block this frame runs was written in.
    pub fn written_in(&self) -> Option<&str> {
        self.written_in.as_deref()
    }

    /// `written_in`, shared rather than borrowed.
    pub fn written_in_shared(&self) -> Option<&Rc<str>> {
        self.written_in.as_ref()
    }

    /// `name`, shared rather than borrowed.
    pub fn shared_name(&self) -> Rc<str> {
        Rc::clone(&self.name)
    }

    /// Say which module the running method was defined in, by class path.
    pub fn owned_by(mut self, owner_path: Option<impl Into<Rc<str>>>) -> Self {
        self.owner_path = owner_path.map(Into::into);
        self
    }

    pub fn owned_by_class(mut self, owner_class: Option<Rc<crate::class::Class>>) -> Self {
        self.owner_class = owner_class;
        self
    }

    /// The class or module the method a block frame was written in was
    /// defined on.
    pub fn owner_class(&self) -> Option<&Rc<crate::class::Class>> {
        self.owner_class.as_ref()
    }

    /// The class path of the module the running method was defined in.
    pub fn owner_path(&self) -> Option<&str> {
        self.owner_path.as_deref()
    }

    /// `owner_path`, shared rather than borrowed.
    pub fn shared_owner_path(&self) -> Option<&Rc<str>> {
        self.owner_path.as_ref()
    }

    /// Record the scope that was current when the frame was pushed.
    pub(crate) fn entered_from(&mut self, scope: Rc<RefCell<Scope>>) {
        self.entering_scope = Some(scope);
    }

    /// The scope that was current when the frame was pushed, which is the
    /// scope of the code that made the call.
    pub(crate) fn entering_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        self.entering_scope.clone()
    }
}
