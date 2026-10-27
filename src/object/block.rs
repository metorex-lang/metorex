// BlockStatement - represents closures/lambdas with captured variables

use crate::ast::Statement;
use crate::callable::Callable;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::vm::VirtualMachine;
use std::rc::Rc;

use super::Object;

pub use crate::ast::parameter_marks::{
    BLOCK_LOCAL_PREFIX, DESTRUCTURED_GROUP_PREFIX, IMPLICIT_IT_PARAM, KEYWORD_PARAM_PREFIX,
    NO_KEYWORDS_PARAM, TRAILING_COMMA_PARAM,
};

/// The name a parameter carries when the method it belongs to is one the
/// interpreter answers itself. Ruby reports the kind of such a parameter and
/// no name at all, since there is no source to have named it.
pub const UNNAMED_PARAMETER: &str = "__mx_unnamed";

/// Places in the source, each the file and the line some code opened on.
pub type CodePlaces = Rc<Vec<(String, usize)>>;

thread_local! {
    static NO_NAMES: Rc<Vec<String>> = Rc::new(Vec::new());
    static NO_OUTER_LOCALS: Rc<Vec<(String, bool)>> = Rc::new(Vec::new());
    static NO_PLACES: CodePlaces = Rc::new(Vec::new());
}

/// An empty list of names, shared rather than allocated for each block.
fn no_names() -> Rc<Vec<String>> {
    NO_NAMES.with(Rc::clone)
}

fn no_outer_locals() -> Rc<Vec<(String, bool)>> {
    NO_OUTER_LOCALS.with(Rc::clone)
}

fn no_places() -> CodePlaces {
    NO_PLACES.with(Rc::clone)
}

/// Block/lambda/closure with captured variables. What it shares with the
/// other blocks its literal made in the same place is read through it from
/// its `BlockOrigin`, which a change copies first when another block holds
/// it too.
#[derive(Debug, Clone)]
pub struct BlockStatement {
    origin: Rc<BlockOrigin>,
    /// Captured variables from outer scope (shared mutable references),
    /// shared by every call of the block rather than copied into each.
    pub captured_vars: crate::scope::CapturedNames,
    /// The locals listed where the block was written, in the order listed
    /// there, including the ones assigned after the block, which
    /// `local_variables` in the block lists though the block cannot read them.
    pub reserved_names: Rc<Vec<String>>,
    /// True for `-> {}` and `lambda {}`, false for `proc {}` and every
    /// ordinary block. Lambdas check arity strictly; procs pad missing
    /// arguments with nil and drop extras.
    pub is_lambda: bool,
    /// The method invocation this block was written inside. A `return` in the
    /// body unwinds to that invocation, however many other methods the block
    /// travels through first. None for a block created outside any method.
    pub home_frame: Option<u64>,
    /// What every copy of the block shares and any of them changes.
    pub flags: Rc<BlockFlags>,
}

/// The state every copy of a block shares, so a change made through one
/// copy reaches a `dup` of it.
#[derive(Debug)]
pub struct BlockFlags {
    /// Whether the call the block was attached to is still running. A
    /// `break` returns from that call, so once it has returned a `break`
    /// raises LocalJumpError.
    pub attached_call_running: std::cell::Cell<bool>,
    /// Whether `ruby2_keywords` was called on the Proc this block stands for,
    /// so a trailing keyword hash gathered into its splat is marked.
    pub ruby2_keywords: std::cell::Cell<bool>,
}

/// What a block was written as and where it was made, which the blocks a
/// literal makes in the same place share.
#[derive(Debug, Clone)]
pub struct BlockOrigin {
    /// Parameter names
    pub parameters: Vec<String>,
    /// The locals of the scope the block was written in that its body names.
    pub outer_locals: Rc<Vec<(String, bool)>>,
    /// Default values for optional parameters, keyed by index into
    /// `parameters` (e.g. `{ |a, b = 1| }` records `(1, <1>)`).
    pub parameter_defaults: Vec<(usize, crate::ast::Expression)>,
    /// Block body (AST statements), shared by every block a literal makes.
    pub body: crate::ast::Body,
    /// Lexical class/module nesting at the moment the block was defined.
    /// Restored during invocation so a bare `Foo = 1` inside the body lands
    /// on the same enclosing module that an unbroken straight-line statement
    /// would have hit.
    pub captured_def_scope: Vec<Rc<Class>>,
    /// Where a constant written in the body lands, when the block was written
    /// in the body of a block run as a class body, as `class_exec` runs one:
    /// the scope that block was written in, however the block is later run.
    pub captured_constant_home: Option<Option<Rc<Class>>>,
    /// The class or module a class variable in the body belongs to, as it
    /// stood where the block was made, or none at the top level. Unset for
    /// a block no literal made, whose home comes from its nesting.
    pub class_var_home: Option<Option<Rc<Class>>>,
    /// The scopes open where the block was written, innermost first, which is
    /// what `Module.nesting` in the body reports and where an `eval` written
    /// there opens what it defines.
    pub captured_nesting: crate::object::Nesting,
    /// The method that lexically encloses this block, as the (callee, defined)
    /// pair `__callee__` and `__method__` report. None for a block created
    /// outside any method.
    pub defining_method: Option<(Rc<str>, Rc<str>)>,
    /// The class path of the module the enclosing method was defined in.
    pub defining_owner: Option<Rc<str>>,
    /// The class or module the enclosing method was defined on, which is
    /// where a `super` in the block starts from.
    pub defining_class: Option<Rc<Class>>,
    /// The file the block was written in, which a backtrace entry for a call
    /// made from its body has to name.
    pub source_file: Option<Rc<str>>,
    /// The line the block was opened on, which is where `source_location`
    /// says it was written even when its body starts further down.
    pub opened_at: Option<usize>,
    /// The column the block was opened at, counted in characters from 0,
    /// which tells it from another block opened on the same line.
    pub opened_column: Option<usize>,
    /// The name a callable built from a Symbol stands for, which is what it
    /// says of itself in place of a file and a line.
    pub from_symbol: Option<String>,
    /// The scope the block was written in, which is what a backtrace names it
    /// by however far from there it is called.
    pub written_in: Option<Rc<str>>,
    /// How many blocks deep the block was written, counting itself, which
    /// is the depth a backtrace names it by however it is called.
    pub written_depth: Option<u32>,
    /// The places the block was written within, outermost first and ending
    /// with its own, each as the file and the line the code opened on. A
    /// trace aimed at a method or a block fires for the blocks written
    /// inside it by finding the target among these.
    pub written_within: CodePlaces,
    /// What the body compiled to, once it has run on the bytecode engine.
    pub unit: crate::vm::units::UnitSlot,
}

impl std::ops::Deref for BlockStatement {
    type Target = BlockOrigin;

    fn deref(&self) -> &BlockOrigin {
        &self.origin
    }
}

impl std::ops::DerefMut for BlockStatement {
    fn deref_mut(&mut self) -> &mut BlockOrigin {
        Rc::make_mut(&mut self.origin)
    }
}

/// Two blocks are the same when they were written the same way. The captured
/// variables are left out: a block can close over itself, so comparing them
/// would not terminate.
impl PartialEq for BlockStatement {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
            && self.parameter_defaults == other.parameter_defaults
            && self.body == other.body
            && self.defining_method == other.defining_method
            && self.is_lambda == other.is_lambda
    }
}

impl BlockOrigin {
    /// The places of a block written nowhere in the program.
    pub fn no_places() -> CodePlaces {
        no_places()
    }

    /// An origin naming no place, for a block no literal made.
    pub fn unplaced(
        parameters: Vec<String>,
        parameter_defaults: Vec<(usize, crate::ast::Expression)>,
        body: crate::ast::Body,
        captured_def_scope: Vec<Rc<Class>>,
        defining_method: Option<(Rc<str>, Rc<str>)>,
    ) -> Self {
        Self {
            parameters,
            outer_locals: no_outer_locals(),
            parameter_defaults,
            body,
            captured_def_scope,
            captured_constant_home: None,
            class_var_home: None,
            captured_nesting: crate::object::no_nesting(),
            defining_method,
            defining_owner: None,
            defining_class: None,
            source_file: None,
            opened_at: None,
            opened_column: None,
            from_symbol: None,
            written_in: None,
            written_depth: None,
            written_within: no_places(),
            unit: Default::default(),
        }
    }
}

impl BlockStatement {
    /// Create a new block closure
    pub fn new(
        parameters: Vec<String>,
        body: Vec<Statement>,
        captured_vars: crate::scope::Captures,
    ) -> Self {
        Self::with_def_scope(
            parameters,
            Vec::new(),
            body.into(),
            captured_vars,
            Vec::new(),
            None,
            false,
        )
    }

    /// Create a new block closure with a captured lexical scope. Used by
    /// the Lambda evaluator so the block remembers which class/module it
    /// was lexically inside.
    pub fn with_def_scope(
        parameters: Vec<String>,
        parameter_defaults: Vec<(usize, crate::ast::Expression)>,
        body: crate::ast::Body,
        captured_vars: crate::scope::Captures,
        captured_def_scope: Vec<Rc<Class>>,
        defining_method: Option<(Rc<str>, Rc<str>)>,
        is_lambda: bool,
    ) -> Self {
        let origin = BlockOrigin::unplaced(
            parameters,
            parameter_defaults,
            body,
            captured_def_scope,
            defining_method,
        );
        Self::from_origin(Rc::new(origin), Rc::new(captured_vars), is_lambda)
    }

    /// A block made from `origin`, closing over `captured_vars`.
    pub fn from_origin(
        origin: Rc<BlockOrigin>,
        captured_vars: crate::scope::CapturedNames,
        is_lambda: bool,
    ) -> Self {
        Self {
            origin,
            captured_vars,
            reserved_names: no_names(),
            is_lambda,
            home_frame: None,
            flags: Rc::new(BlockFlags {
                attached_call_running: std::cell::Cell::new(true),
                ruby2_keywords: std::cell::Cell::new(false),
            }),
        }
    }

    /// Get the captured variables
    pub fn captured_vars(&self) -> &crate::scope::Captures {
        &self.captured_vars
    }

    /// True when a single array argument spreads across the parameters, which
    /// a block does when it declares more than one of them or when its
    /// parameter list ended in a comma.
    pub fn destructures_single_array(&self) -> bool {
        // A lambda takes its arguments the way a method does, so a lone array
        // stays one argument rather than spreading across the parameters.
        if self.is_lambda {
            return false;
        }
        if self
            .parameters
            .iter()
            .any(|name| name == TRAILING_COMMA_PARAM)
        {
            return true;
        }
        // Only the positional parameters count: a block taking `|*a, **kw|`
        // has one place for its arguments, so a lone array stays whole. A
        // block-local after the `;` takes no argument.
        self.parameters
            .iter()
            .filter(|name| {
                !name.starts_with('&')
                    && !name.starts_with(KEYWORD_PARAM_PREFIX)
                    && !name.starts_with("**")
                    && !name.starts_with(BLOCK_LOCAL_PREFIX)
            })
            .count()
            > 1
    }

    /// Parameters that actually bind a name, excluding the trailing-comma marker.
    pub fn binding_parameters(&self) -> std::borrow::Cow<'_, [String]> {
        let binds = |p: &&String| {
            *p != TRAILING_COMMA_PARAM
                && *p != NO_KEYWORDS_PARAM
                && !p.starts_with(BLOCK_LOCAL_PREFIX)
        };
        if self.parameters.iter().all(|p| binds(&p)) {
            return std::borrow::Cow::Borrowed(&self.parameters);
        }
        self.parameters.iter().filter(binds).cloned().collect()
    }

    /// The names written after the `;`, which are locals of the block rather
    /// than parameters.
    pub fn block_locals(&self) -> Vec<String> {
        self.parameters
            .iter()
            .filter_map(|p| p.strip_prefix(BLOCK_LOCAL_PREFIX))
            .map(|name| name.to_string())
            .collect()
    }

    /// Invoke the block within the provided virtual machine context.
    pub fn call(
        &self,
        vm: &mut VirtualMachine,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        vm.execute_block_callable(self, arguments, position)
    }
}

impl Callable for BlockStatement {
    fn name(&self) -> &str {
        "<block>"
    }

    fn parameters(&self) -> &[String] {
        &self.parameters
    }

    fn body(&self) -> &[Statement] {
        &self.body
    }
}
