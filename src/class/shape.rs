// What a class is made of.

use super::*;

/// A method lookup's answer, with the method state it was found under.
/// What a lookup found, the method state it held under, and how many times
/// the ancestry was walked for the name rather than the cache answering.
pub(crate) type CachedLookup = (u64, Option<Rc<Method>>, u32);

/// The modules mixed into a class in lookup order, with the method state they
/// were worked out under.
pub(crate) type LinearMixins = (u64, Rc<[Rc<Class>]>);

/// Runtime class definition with method table and inheritance.
pub struct Class {
    pub(crate) name: String,
    /// Ruby-visible name, set the first time an anonymous class is assigned
    /// to a constant. `name()` still returns the original (immutable) name —
    /// use `ruby_name()` to pick up the assigned override.
    pub(crate) assigned_name: RefCell<Option<String>>,
    /// Name set by `Module#set_temporary_name`. It takes precedence over the
    /// assigned name until the module gains a permanent one.
    pub(crate) temporary_name: RefCell<Option<String>>,
    /// The subclass of Module this module was made from, for a module
    /// `new` made on one, which is what its `class` answers.
    pub(crate) instance_of: RefCell<Option<Rc<Class>>>,
    pub(crate) superclass: Option<Rc<Class>>,
    pub(crate) methods: RefCell<crate::fast_hash::NameMap<String, Rc<Method>>>,
    pub(crate) instance_variables: RefCell<crate::fast_hash::NameSet<String>>,
    /// Class variables, constants, and class-level bookkeeping keyed by name.
    /// Insertion-ordered so `Module#class_variables` reports definition order.
    pub(crate) class_variables:
        RefCell<crate::fast_hash::NameIndexMap<String, crate::object::Object>>,
    /// Included modules, in reverse inclusion order (last included = first searched).
    pub(crate) mixins: RefCell<Vec<Rc<Class>>>,
    /// Prepended modules, in reverse prepend order. These sit ahead of the
    /// class's own methods, so one of them shadows a same-named method here
    /// and the class's copy is what `super` reaches from it.
    pub(crate) prepends: RefCell<Vec<Rc<Class>>>,
    /// Names of methods whose visibility has been set to private.
    pub(crate) private_method_names: RefCell<crate::fast_hash::NameSet<String>>,
    /// Names of methods whose visibility has been set to protected. Like
    /// private, these cannot be called with an explicit receiver from
    /// outside, but they are reported as instance methods rather than
    /// private ones.
    pub(crate) protected_method_names: RefCell<crate::fast_hash::NameSet<String>>,
    /// Methods explicitly marked public on this class via `public :name` —
    /// distinct from "no entry" so we can override an inherited private
    /// without adding the method itself to this class's method table.
    pub(crate) public_overrides: RefCell<crate::fast_hash::NameSet<String>>,
    /// Lazily-allocated singleton class attached to *this class object* — the
    /// thing `class << SomeClass; end` opens. Holds class-level (def self.x)
    /// methods once we start tracking them as a real class.
    pub(crate) singleton_class: RefCell<Option<Rc<Class>>>,
    /// Direct subclasses (weak refs to avoid keeping garbage subclasses alive).
    pub(crate) subclasses: RefCell<Vec<Weak<Class>>>,
    /// Whether this object is a module rather than a class. Modules and
    /// classes share this representation, and error messages name which one
    /// the receiver is.
    pub(crate) module_flag: std::cell::Cell<bool>,
    /// Frozen flag — once true, mutating methods (alias_method, define_method,
    /// include, etc.) raise FrozenError.
    pub(crate) frozen: std::cell::Cell<bool>,
    /// Current visibility state set by bare `private`/`public`/`protected`
    /// inside the class body. Methods defined while this is `private` are
    /// auto-marked as private. `protected` is treated like `private` for now.
    pub(crate) current_visibility: RefCell<String>,
    /// `autoload :Const, "path"` registry. Maps constant name → unresolved
    /// path string (the value Ruby's `autoload?` returns verbatim).
    pub(crate) autoloads: RefCell<crate::fast_hash::NameMap<String, String>>,
    /// Constant names that *were* registered as autoloads, fired, and
    /// completed loading without actually defining the constant. MRI keeps
    /// these visible in `Module#constants` for a while afterward (so the
    /// name persists as an "unrealized" constant) even though `autoload?`
    /// and `const_defined?` both report nothing. Storing them separately
    /// here keeps `#constants` in sync without resurrecting the autoload.
    pub(crate) unrealized_autoloads: RefCell<crate::fast_hash::NameSet<String>>,
    /// Constant names marked private via `Module#private_constant`. When
    /// any of these is referenced via qualified `Mod::Const` access from
    /// outside the module's lexical scope, MRI raises NameError /private
    /// constant/.
    pub(crate) private_constants: RefCell<crate::fast_hash::NameSet<String>>,
    /// Constant names marked deprecated via `Module#deprecate_constant`.
    /// Reading one emits a "constant Mod::Const is deprecated" warning.
    pub(crate) deprecated_constants: RefCell<crate::fast_hash::NameSet<String>>,
    /// Source location (file, line) of each `autoload :Const, ...` call.
    /// Returned by `Module#const_source_location` when an autoload is
    /// pending or when another thread asks during a load.
    pub(crate) autoload_locations: RefCell<crate::fast_hash::NameMap<String, (String, i64)>>,
    /// Source location (file, line) of each constant assignment that
    /// produced a class_var on this class — `class Foo; X = 1; end`,
    /// `class Foo; class Bar; end; end`, etc. Returned by
    /// `Module#const_source_location` once the constant is bound.
    pub(crate) const_locations: RefCell<crate::fast_hash::NameMap<String, (String, i64)>>,
    /// What `find_method` found for each name, with the method state it was
    /// found under. An entry from an earlier state is looked up again.
    /// What each name's lookup found, keyed by the name as every class's
    /// cache shares it.
    pub(crate) method_cache: RefCell<crate::fast_hash::NameMap<Rc<str>, CachedLookup>>,
    /// What `linear_mixins` answered, with the method state it was worked
    /// out under, which a change to any mixin chain moves on.
    pub(crate) linear_mixins_cache: RefCell<Option<LinearMixins>>,
    /// Whether the method table holds a `def self.name`, kept under a
    /// `__class__` name, which spares a lookup of one building that name
    /// when there is none.
    pub(crate) holds_class_methods: std::cell::Cell<bool>,
    /// Whether a module this extended left a copy of one of its methods
    /// here, under an `__ext__` name.
    pub(crate) holds_extended_copies: std::cell::Cell<bool>,
}

/// A class is written out by name alone. Following what it holds would go on
/// forever, since each of its methods carries the class it was defined in.
impl std::fmt::Debug for Class {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("Class").field("name", &self.name).finish()
    }
}
