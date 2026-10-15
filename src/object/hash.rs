// ObjectHash - wrapper for making Objects hashable for use in a Set

use super::Object;
use crate::object::View;

/// Wrapper that makes an Object usable as a set element. The rendered
/// `hash_value` is what decides membership, and the element itself travels
/// alongside so the set gives back what was put in.
#[derive(Debug, Clone)]
pub struct ObjectHash {
    /// String representation of the object for hashing
    pub(crate) hash_value: String,
    /// The element itself, which `to_a` and `each` hand back.
    pub(crate) value: Object,
}

impl PartialEq for ObjectHash {
    fn eq(&self, other: &Self) -> bool {
        self.hash_value == other.hash_value
    }
}

impl Eq for ObjectHash {}

impl std::hash::Hash for ObjectHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash_value.hash(state);
    }
}

impl ObjectHash {
    /// Create a hashable wrapper from an object. Anything with a stable
    /// rendering can be an element; an Instance is told apart by identity,
    /// which is what its rendering carries.
    pub fn from_object(obj: &Object) -> Option<Self> {
        let hash_value = match obj.view() {
            View::Nil => "nil".to_string(),
            View::Bool(value) => value.to_string(),
            View::Int(value) => value.to_string(),
            // A Float keeps its fraction, so 1.0 and 1 are different
            // elements the way `eql?` makes them.
            View::Float(value) if value.fract() == 0.0 && value.is_finite() => {
                format!("{:.1}", value)
            }
            View::Float(value) => value.to_string(),
            View::String(text) => format!("\"{}\"", text),
            View::Symbol(name) => format!(":{}", name),
            View::BigInt(value) => value.to_string(),
            // A collection is told apart by what it holds, which its own
            // rendering reports. That rendering guards against a collection
            // that reaches itself, where a structural walk would not.
            View::Array(_) | View::Dict(_) | View::Range { .. } | View::Regex(_, _) => {
                format!("{}{}", obj.type_name(), obj)
            }
            View::Instance(instance) => {
                format!("#<{:p}>", std::rc::Rc::as_ptr(instance))
            }
            // A class, a module, an exception and a callable are each told
            // apart by which object they are.
            View::Class(class) | View::Module(class) => {
                format!("#<{:p}>", std::rc::Rc::as_ptr(class))
            }
            View::Exception(details) => format!("#<{:p}>", std::rc::Rc::as_ptr(details)),
            View::Block(block) => format!("#<{:p}>", std::rc::Rc::as_ptr(block)),
            View::Method(method) => format!("#<{:p}>", std::rc::Rc::as_ptr(method)),
            View::Binding(binding) => format!("#<{:p}>", std::rc::Rc::as_ptr(binding)),
            // A Set is told apart by what it holds, in an order-independent
            // way, so two sets with the same elements are one element.
            View::Set(set) => {
                let mut keys: Vec<String> = set
                    .borrow()
                    .iter()
                    .map(|held| held.hash_value.clone())
                    .collect();
                keys.sort();
                format!("Set[{}]", keys.join(", "))
            }
            _ => return None,
        };
        Some(Self {
            hash_value,
            value: obj.clone(),
        })
    }

    /// A hashable wrapper that tells elements apart by which object they are
    /// rather than by what they hold, which is what a set comparing by
    /// identity puts its elements under.
    pub fn by_identity(obj: &Object) -> Option<Self> {
        let hash_value = match obj.view() {
            View::String(text) => format!("@{:p}", std::rc::Rc::as_ptr(text)),
            View::Array(elements) => format!("@{:p}", std::rc::Rc::as_ptr(elements)),
            View::Dict(pairs) => format!("@{:p}", std::rc::Rc::as_ptr(pairs)),
            View::Set(elements) => format!("@{:p}", std::rc::Rc::as_ptr(elements)),
            View::Block(block) => format!("@{:p}", std::rc::Rc::as_ptr(block)),
            View::Method(method) => format!("@{:p}", std::rc::Rc::as_ptr(method)),
            // Everything else stands for one value however many names reach
            // it, so identity and equality are the same question for it.
            _ => return Self::from_object(obj),
        };
        Some(Self {
            hash_value,
            value: obj.clone(),
        })
    }
}
