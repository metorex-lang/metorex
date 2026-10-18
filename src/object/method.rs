// Method struct - represents a class method (bound or unbound)

use crate::ast::{Expression, Statement};
use crate::callable::Callable;
use crate::error::SourceLocation;
use std::cell::RefCell;
use std::rc::Rc;

use super::Object;

/// The names a method's frames carry, made once per kind of receiver: index
/// 0 for an object, 1 for a class or module. A copy starts empty, since a
/// copy is usually renamed or moved to another owner.
#[derive(Debug, Default)]
pub struct FrameNames(pub RefCell<[Option<CachedFrameName>; 2]>);

/// One cached frame name: the name the method was called by, the frame name,
/// and the name it was defined as.
pub type CachedFrameName = (Rc<str>, Rc<str>, Rc<str>);

/// What a method's body says about the method, worked out on its first
/// call. A copy starts empty, as `FrameNames` does.
#[derive(Debug, Default)]
pub struct BodyFacts {
    places: std::cell::OnceCell<crate::object::CodePlaces>,
    owner_path: std::cell::OnceCell<Rc<str>>,
}

impl Clone for BodyFacts {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl BodyFacts {
    /// The class path of `owner`, the class the method was defined in, or
    /// None for an anonymous one. A permanent name is kept after the first
    /// call, since it cannot change.
    pub fn owner_path(&self, owner: &crate::class::Class) -> Option<Rc<str>> {
        if let Some(path) = self.owner_path.get() {
            return Some(Rc::clone(path));
        }
        let path = owner.ruby_name();
        if path.is_empty() {
            return None;
        }
        let path: Rc<str> = path.into();
        if owner.has_permanent_name() {
            let _ = self.owner_path.set(Rc::clone(&path));
        }
        Some(path)
    }

    /// The places the method's body was written within: its file and line,
    /// when it has a file.
    pub fn places(&self, method: &Method) -> crate::object::CodePlaces {
        Rc::clone(self.places.get_or_init(|| {
            Rc::new(
                method
                    .source_location
                    .as_ref()
                    .and_then(|written| Some((written.filename.clone()?, written.line)))
                    .into_iter()
                    .collect(),
            )
        }))
    }
}

impl Clone for FrameNames {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// The class of the last receiver whose native methods were found to have
/// none by a method's name, with the method state that held then. While the
/// state holds, the native methods of that class are not looked through
/// again before the method's body runs. A copy starts empty, as
/// `FrameNames` does.
#[derive(Debug, Default)]
pub struct NativeMiss(RefCell<Option<(std::rc::Weak<crate::class::Class>, u64)>>);

impl NativeMiss {
    /// Whether no native method answered this name for `class` under the
    /// method state `state`.
    pub fn recorded(&self, class: &Rc<crate::class::Class>, state: u64) -> bool {
        self.0.borrow().as_ref().is_some_and(|(missed, under)| {
            *under == state && std::ptr::eq(missed.as_ptr(), Rc::as_ptr(class))
        })
    }

    /// Record that no native method answered this name for `class` under
    /// the method state `state`.
    pub fn record(&self, class: &Rc<crate::class::Class>, state: u64) {
        *self.0.borrow_mut() = Some((Rc::downgrade(class), state));
    }
}

impl Clone for NativeMiss {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// Method definition (function bound to a class)
#[derive(Debug, Clone)]
pub struct Method {
    /// Name of the method
    pub name: String,
    /// Positional parameter names (excludes &block and keyword params)
    pub parameters: Vec<String>,
    /// Default values for positional parameters, indexed by position
    pub default_parameters: Vec<(usize, Expression)>,
    /// Named keyword parameters: (name, optional_default_expression)
    /// e.g., `def f(name:, age: 10)` → [("name", None), ("age", Some(IntLiteral(10)))]
    pub keyword_parameters: Vec<(String, Option<Expression>)>,
    /// Keyword-rest parameter name from `**kwargs`, when declared.
    pub keyword_rest_parameter: Option<String>,
    /// The name this method was defined under, when it is reachable by a
    /// different name because `alias_method` renamed a copy.
    pub original_name: Option<String>,
    /// The native method this is a stub for. `alias_method` copying a method
    /// that lives in the native tables has no body to copy, so the stub
    /// records the name to dispatch under instead.
    pub native_alias: Option<String>,
    /// Optional block parameter name (from `&block` syntax)
    pub block_parameter: Option<String>,
    /// Variadic (splat) parameter: (positional_index, name) for `*args`
    pub variadic_param: Option<(usize, String)>,
    /// Method body (AST statements)
    pub body: crate::ast::Body,
    /// Optional receiver (for bound methods)
    pub receiver: Option<Box<Object>>,
    /// Receiver this method must run against no matter where it is installed.
    /// Set by `Method#to_proc`, which produces a Proc permanently attached to
    /// the object the Method was extracted from.
    pub bound_self: Option<Box<Object>>,
    /// Owner of the method (class name or "main" for top-level functions)
    pub owner: Option<String>,
    /// Owning class/module object, when known. Needed by `define_method` to
    /// validate that an UnboundMethod may be rebound onto the target module,
    /// and by `Method#owner` to return the module itself rather than its name.
    pub owner_class: Option<Rc<crate::class::Class>>,
    /// The class the method was looked up on, which may sit below the module
    /// that owns it. `super_method` needs it to walk the same ancestors the
    /// lookup did.
    pub origin_class: Option<Rc<crate::class::Class>>,
    /// Source location where the method is defined
    pub source_location: Option<SourceLocation>,
    /// The line the definition's `end` sits on, which a `return` event
    /// names when the body runs to its end.
    pub end_line: Option<usize>,
    /// Captured closure variables (from define_method blocks)
    pub captured_vars: Option<indexmap::IndexMap<String, Rc<RefCell<Object>>>>,
    /// True if this method was undefined via undef_method (calling it raises an error)
    pub is_undefined: bool,
    /// Lexical class/module nesting captured from the Proc this method was
    /// built from. Restored while the body runs so a nested `def` lands in the
    /// Proc's default definee rather than the receiving class.
    pub captured_def_scope: Vec<Rc<crate::class::Class>>,
    /// True when the body came from a block or Proc handed to `define_method`.
    /// Those bodies follow lambda control flow: `break` and `next` return a
    /// value, and `redo` re-runs the body.
    pub lambda_body: bool,
    /// Refinement modules active (lexically) when this method was defined,
    /// each paired with the snapshot of refined-class names at that moment.
    pub captured_refinements: Vec<(Rc<crate::class::Class>, Vec<String>)>,
    /// For a copy `alias_method` made, the class or module the body was
    /// written in. `super` from the copy starts past that one, whatever
    /// class the copy was installed on.
    pub alias_origin: Option<Rc<crate::class::Class>>,
    /// The frame name and defined name a call of this method runs under.
    pub frame_names: FrameNames,
    pub native_miss: NativeMiss,
    pub body_facts: BodyFacts,
    /// The lexically enclosing modules at the point of definition, innermost
    /// first. `Module.nesting` inside the method reports these rather than
    /// the scopes open at the call site.
    pub captured_nesting: crate::object::Nesting,
    /// True when `Method#to_proc` made this, which Ruby reports as a Proc
    /// rather than as the Method it came from.
    pub reads_as_proc: bool,
    /// Whether `ruby2_keywords` named this method, so a trailing keyword hash
    /// gathered into its splat is marked and can be passed on as keywords.
    /// The flag is shared with every copy, which is what makes it reach an
    /// alias made either before or after the name was given.
    pub ruby2_keywords: Rc<std::cell::Cell<bool>>,
    /// The class or module a `def` installed the method in. A `def` run
    /// while the method runs installs there too.
    pub definee: Option<Rc<crate::class::Class>>,
    /// The C function a C extension defined this method with.
    pub c_function: Option<crate::vm::capi::CFunction>,
    /// What the body compiled to, once it has run on the bytecode engine.
    pub unit: crate::vm::units::UnitSlot,
}

impl Method {
    /// Create a new method
    pub fn new(name: String, parameters: Vec<String>, body: impl Into<crate::ast::Body>) -> Self {
        let body = body.into();
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            keyword_rest_parameter: None,
            original_name: None,
            native_alias: None,
            block_parameter: None,
            variadic_param: None,
            body,
            receiver: None,
            bound_self: None,
            owner: None,
            owner_class: None,
            origin_class: None,
            source_location: None,
            end_line: None,
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            alias_origin: None,
            frame_names: FrameNames::default(),
            native_miss: NativeMiss::default(),
            body_facts: BodyFacts::default(),
            captured_nesting: crate::object::no_nesting(),
            reads_as_proc: false,
            ruby2_keywords: Rc::new(std::cell::Cell::new(false)),
            definee: None,
            c_function: None,
            unit: Default::default(),
        }
    }

    /// Create a new method with an owner
    pub fn with_owner(
        name: String,
        parameters: Vec<String>,
        body: impl Into<crate::ast::Body>,
        owner: String,
    ) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            keyword_rest_parameter: None,
            original_name: None,
            native_alias: None,
            block_parameter: None,
            variadic_param: None,
            body: body.into(),
            receiver: None,
            bound_self: None,
            owner: Some(owner),
            owner_class: None,
            origin_class: None,
            source_location: None,
            end_line: None,
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            alias_origin: None,
            frame_names: FrameNames::default(),
            native_miss: NativeMiss::default(),
            body_facts: BodyFacts::default(),
            captured_nesting: crate::object::no_nesting(),
            reads_as_proc: false,
            ruby2_keywords: Rc::new(std::cell::Cell::new(false)),
            definee: None,
            c_function: None,
            unit: Default::default(),
        }
    }

    /// Create a new method with a source location
    pub fn with_source_location(
        name: String,
        parameters: Vec<String>,
        body: impl Into<crate::ast::Body>,
        source_location: SourceLocation,
    ) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            keyword_rest_parameter: None,
            original_name: None,
            native_alias: None,
            block_parameter: None,
            variadic_param: None,
            body: body.into(),
            receiver: None,
            bound_self: None,
            owner: None,
            owner_class: None,
            origin_class: None,
            source_location: Some(source_location),
            end_line: None,
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            alias_origin: None,
            frame_names: FrameNames::default(),
            native_miss: NativeMiss::default(),
            body_facts: BodyFacts::default(),
            captured_nesting: crate::object::no_nesting(),
            reads_as_proc: false,
            ruby2_keywords: Rc::new(std::cell::Cell::new(false)),
            definee: None,
            c_function: None,
            unit: Default::default(),
        }
    }

    /// Create a new method with both owner and source location
    pub fn with_owner_and_location(
        name: String,
        parameters: Vec<String>,
        body: impl Into<crate::ast::Body>,
        owner: String,
        source_location: SourceLocation,
    ) -> Self {
        Self {
            name,
            parameters,
            default_parameters: vec![],
            keyword_parameters: vec![],
            keyword_rest_parameter: None,
            original_name: None,
            native_alias: None,
            block_parameter: None,
            variadic_param: None,
            body: body.into(),
            receiver: None,
            bound_self: None,
            owner: Some(owner),
            owner_class: None,
            origin_class: None,
            source_location: Some(source_location),
            end_line: None,
            captured_vars: None,
            is_undefined: false,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            alias_origin: None,
            frame_names: FrameNames::default(),
            native_miss: NativeMiss::default(),
            body_facts: BodyFacts::default(),
            captured_nesting: crate::object::no_nesting(),
            reads_as_proc: false,
            ruby2_keywords: Rc::new(std::cell::Cell::new(false)),
            definee: None,
            c_function: None,
            unit: Default::default(),
        }
    }

    /// Create an undefined method sentinel (for undef_method).
    pub fn undefined(name: String) -> Self {
        Self {
            name,
            parameters: vec![],
            default_parameters: vec![],
            keyword_parameters: vec![],
            keyword_rest_parameter: None,
            original_name: None,
            native_alias: None,
            block_parameter: None,
            variadic_param: None,
            body: crate::ast::Body::from([]),
            receiver: None,
            bound_self: None,
            owner: None,
            owner_class: None,
            origin_class: None,
            source_location: None,
            end_line: None,
            captured_vars: None,
            is_undefined: true,
            lambda_body: false,
            captured_def_scope: Vec::new(),
            captured_refinements: Vec::new(),
            alias_origin: None,
            frame_names: FrameNames::default(),
            native_miss: NativeMiss::default(),
            body_facts: BodyFacts::default(),
            captured_nesting: crate::object::no_nesting(),
            reads_as_proc: false,
            ruby2_keywords: Rc::new(std::cell::Cell::new(false)),
            definee: None,
            c_function: None,
            unit: Default::default(),
        }
    }

    /// Bind this method to a receiver
    pub fn bind(&self, receiver: Object) -> Self {
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            default_parameters: self.default_parameters.clone(),
            keyword_parameters: self.keyword_parameters.clone(),
            keyword_rest_parameter: self.keyword_rest_parameter.clone(),
            original_name: self.original_name.clone(),
            native_alias: self.native_alias.clone(),
            block_parameter: self.block_parameter.clone(),
            variadic_param: self.variadic_param.clone(),
            body: self.body.clone(),
            receiver: Some(Box::new(receiver)),
            bound_self: self.bound_self.clone(),
            owner: self.owner.clone(),
            owner_class: self.owner_class.clone(),
            origin_class: self.origin_class.clone(),
            source_location: self.source_location.clone(),
            end_line: None,
            captured_vars: self.captured_vars.clone(),
            is_undefined: self.is_undefined,
            lambda_body: self.lambda_body,
            captured_def_scope: self.captured_def_scope.clone(),
            captured_refinements: self.captured_refinements.clone(),
            alias_origin: self.alias_origin.clone(),
            frame_names: FrameNames::default(),
            native_miss: NativeMiss::default(),
            body_facts: BodyFacts::default(),
            captured_nesting: self.captured_nesting.clone(),
            reads_as_proc: self.reads_as_proc,
            ruby2_keywords: Rc::clone(&self.ruby2_keywords),
            definee: self.definee.clone(),
            c_function: self.c_function,
            unit: self.unit.clone(),
        }
    }

    /// Check if this method is bound to a receiver
    pub fn is_bound(&self) -> bool {
        self.receiver.is_some()
    }

    /// Get the receiver if this method is bound
    pub fn receiver(&self) -> Option<&Object> {
        self.receiver.as_deref()
    }
}

/// `Class` has no structural equality, so the owning module is compared by
/// identity while every other field compares structurally.
impl PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        let same_owner_class = match (&self.owner_class, &other.owner_class) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };
        let same_refinements = self.captured_refinements.len() == other.captured_refinements.len()
            && self
                .captured_refinements
                .iter()
                .zip(other.captured_refinements.iter())
                .all(|((a_mod, a_names), (b_mod, b_names))| {
                    Rc::ptr_eq(a_mod, b_mod) && a_names == b_names
                });
        // An alias and the method it was made from are the same definition,
        // so they compare by the name each was defined under.
        let defined_name = |method: &Method| {
            method
                .original_name
                .clone()
                .unwrap_or_else(|| method.name.clone())
        };
        same_owner_class
            && same_refinements
            && defined_name(self) == defined_name(other)
            && self.parameters == other.parameters
            && self.default_parameters == other.default_parameters
            && self.keyword_parameters == other.keyword_parameters
            && self.keyword_rest_parameter == other.keyword_rest_parameter
            && self.block_parameter == other.block_parameter
            && self.variadic_param == other.variadic_param
            && self.body == other.body
            && self.receiver == other.receiver
            && self.bound_self == other.bound_self
            && self.owner == other.owner
            && self.source_location == other.source_location
            && self.captured_vars == other.captured_vars
            && self.is_undefined == other.is_undefined
            && self.lambda_body == other.lambda_body
            && self.captured_def_scope.len() == other.captured_def_scope.len()
            && self
                .captured_def_scope
                .iter()
                .zip(other.captured_def_scope.iter())
                .all(|(a, b)| Rc::ptr_eq(a, b))
    }
}

impl Callable for Method {
    fn name(&self) -> &str {
        &self.name
    }

    fn parameters(&self) -> &[String] {
        &self.parameters
    }

    fn body(&self) -> &[Statement] {
        &self.body
    }
}
