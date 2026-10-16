// Trial deletion over the listed heap objects, which frees the cycles
// counting cannot: objects that refer to each other and that nothing outside
// them refers to.
//
// Each object's count is compared with the references the listed objects and
// the parts they hold account for. An object whose count is higher is held by
// something the collector cannot see, such as a Rust local or a structure it
// does not walk, and it is kept with everything it reaches. A reference the
// walk misses only keeps an object alive, so every part walked must report
// each reference it holds exactly once.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use super::live;
use super::{Binding, BlockStatement, Exception, Instance, Method, Object, ObjectHash, View};

type Variable = Rc<RefCell<Object>>;

/// Something that holds references: a listed object, or a part of one that
/// several objects can share.
enum Node {
    /// The cell a pointer value points at, which its copies share.
    Cell(Object),
    Array(Rc<RefCell<Vec<Object>>>),
    Hash(Rc<RefCell<IndexMap<String, Object>>>),
    Set(Rc<RefCell<IndexSet<ObjectHash>>>),
    Instance(Rc<RefCell<Instance>>),
    Exception(Rc<RefCell<Exception>>),
    Block(Rc<BlockStatement>),
    Method(Rc<Method>),
    Binding(Rc<Binding>),
    /// The local variables a block closes over.
    Captured(Rc<IndexMap<String, Variable>>),
    Variable(Variable),
}

fn address<T>(held: &Rc<T>) -> usize {
    Rc::as_ptr(held) as *const () as usize
}

/// The cell a value points at, as a node.
fn value(object: &Object, found: &mut Vec<Node>) {
    if object.is_pointer() {
        found.push(Node::Cell(object.clone()));
    }
}

fn variables<'a>(held: impl IntoIterator<Item = &'a Variable>, found: &mut Vec<Node>) {
    found.extend(held.into_iter().map(|one| Node::Variable(one.clone())));
}

impl Node {
    fn address(&self) -> usize {
        match self {
            Node::Cell(object) => object.bits() as usize,
            Node::Array(held) => address(held),
            Node::Hash(held) => address(held),
            Node::Set(held) => address(held),
            Node::Instance(held) => address(held),
            Node::Exception(held) => address(held),
            Node::Block(held) => address(held),
            Node::Method(held) => address(held),
            Node::Binding(held) => address(held),
            Node::Captured(held) => address(held),
            Node::Variable(held) => address(held),
        }
    }

    /// How many references the node has, the collector's own among them.
    fn count(&self) -> usize {
        match self {
            Node::Cell(object) => object.cell_count(),
            Node::Array(held) => Rc::strong_count(held),
            Node::Hash(held) => Rc::strong_count(held),
            Node::Set(held) => Rc::strong_count(held),
            Node::Instance(held) => Rc::strong_count(held),
            Node::Exception(held) => Rc::strong_count(held),
            Node::Block(held) => Rc::strong_count(held),
            Node::Method(held) => Rc::strong_count(held),
            Node::Binding(held) => Rc::strong_count(held),
            Node::Captured(held) => Rc::strong_count(held),
            Node::Variable(held) => Rc::strong_count(held),
        }
    }

    /// Whether the node is an object the list holds, rather than a part.
    fn is_listed(&self) -> bool {
        !matches!(self, Node::Cell(_) | Node::Captured(_) | Node::Variable(_))
    }

    /// Add a node for each reference this one holds. Answers false when the
    /// node is borrowed for writing, which means something running holds it.
    fn held(&self, found: &mut Vec<Node>) -> bool {
        match self {
            Node::Cell(object) => match object.view() {
                View::Array(held) => found.push(Node::Array(held.clone())),
                View::Dict(held) => found.push(Node::Hash(held.clone())),
                View::Set(held) => found.push(Node::Set(held.clone())),
                View::Instance(held) => found.push(Node::Instance(held.clone())),
                View::Exception(held) => found.push(Node::Exception(held.clone())),
                View::Block(held) => found.push(Node::Block(held.clone())),
                View::Method(held) => found.push(Node::Method(held.clone())),
                View::Binding(held) => found.push(Node::Binding(held.clone())),
                View::Range { start, end, .. } => {
                    value(start, found);
                    value(end, found);
                }
                View::Result(Ok(held) | Err(held)) => value(held, found),
                _ => {}
            },
            Node::Array(held) => {
                let Ok(elements) = held.try_borrow() else {
                    return false;
                };
                elements.iter().for_each(|one| value(one, found));
            }
            Node::Hash(held) => {
                let Ok(entries) = held.try_borrow() else {
                    return false;
                };
                entries.values().for_each(|one| value(one, found));
            }
            Node::Set(held) => {
                let Ok(members) = held.try_borrow() else {
                    return false;
                };
                members.iter().for_each(|one| value(&one.value, found));
            }
            Node::Instance(held) => {
                let Ok(instance) = held.try_borrow() else {
                    return false;
                };
                instance
                    .instance_vars
                    .values()
                    .for_each(|one| value(one, found));
            }
            Node::Exception(held) => {
                let Ok(exception) = held.try_borrow() else {
                    return false;
                };
                for boxed in [&exception.cause, &exception.receiver]
                    .into_iter()
                    .flatten()
                {
                    value(boxed, found);
                }
                for one in [
                    &exception.backtrace_array,
                    &exception.backtrace_locations_array,
                ]
                .into_iter()
                .flatten()
                {
                    value(one, found);
                }
                exception
                    .instance_vars
                    .values()
                    .for_each(|one| value(one, found));
            }
            Node::Block(held) => found.push(Node::Captured(held.captured_vars.clone())),
            Node::Method(held) => {
                for boxed in [&held.receiver, &held.bound_self].into_iter().flatten() {
                    value(boxed, found);
                }
                if let Some(captured) = &held.captured_vars {
                    variables(captured.values(), found);
                }
            }
            Node::Binding(held) => {
                let (Ok(named), Ok(implicit)) = (
                    held.variables.try_borrow(),
                    held.implicit_parameters.try_borrow(),
                ) else {
                    return false;
                };
                variables(named.values(), found);
                variables(implicit.iter().map(|(_, one)| one), found);
                if let Some(receiver) = &held.receiver {
                    value(receiver, found);
                }
            }
            Node::Captured(held) => variables(held.values(), found),
            Node::Variable(held) => {
                let Ok(object) = held.try_borrow() else {
                    return false;
                };
                value(&object, found);
            }
        }
        true
    }

    /// Let go of every reference a node nothing reaches holds, which breaks
    /// the cycle it is part of. What it let go of is handed to `dropped`, to
    /// be freed once every node has let go.
    fn clear(&self, dropped: &mut Vec<Object>, dropped_variables: &mut Vec<Variable>) {
        match self {
            Node::Array(held) => {
                if let Ok(mut elements) = held.try_borrow_mut() {
                    dropped.append(&mut elements);
                }
            }
            Node::Hash(held) => {
                if let Ok(mut entries) = held.try_borrow_mut() {
                    dropped.extend(std::mem::take(&mut *entries).into_values());
                }
            }
            Node::Set(held) => {
                if let Ok(mut members) = held.try_borrow_mut() {
                    dropped.extend(
                        std::mem::take(&mut *members)
                            .into_iter()
                            .map(|one| one.value),
                    );
                }
            }
            Node::Instance(held) => {
                if let Ok(mut instance) = held.try_borrow_mut() {
                    dropped.extend(std::mem::take(&mut instance.instance_vars).into_values());
                }
            }
            Node::Exception(held) => {
                if let Ok(mut exception) = held.try_borrow_mut() {
                    let exception = &mut *exception;
                    for boxed in [exception.cause.take(), exception.receiver.take()]
                        .into_iter()
                        .flatten()
                    {
                        dropped.push(*boxed);
                    }
                    dropped.extend(exception.backtrace_array.take());
                    dropped.extend(exception.backtrace_locations_array.take());
                    dropped.extend(std::mem::take(&mut exception.instance_vars).into_values());
                }
            }
            Node::Binding(held) => {
                if let Ok(mut named) = held.variables.try_borrow_mut() {
                    dropped_variables.extend(std::mem::take(&mut *named).into_values());
                }
                if let Ok(mut implicit) = held.implicit_parameters.try_borrow_mut() {
                    dropped_variables.extend(
                        std::mem::take(&mut *implicit)
                            .into_iter()
                            .map(|(_, one)| one),
                    );
                }
            }
            Node::Variable(held) => {
                if let Ok(mut object) = held.try_borrow_mut() {
                    dropped.push(std::mem::replace(&mut *object, Object::Nil));
                }
            }
            Node::Cell(_) | Node::Block(_) | Node::Method(_) | Node::Captured(_) => {}
        }
    }
}

/// The listed objects that can hold references, as nodes.
fn listed_nodes() -> Vec<Node> {
    live::with_lists(|lists| {
        let mut nodes = Vec::new();
        nodes.extend(lists.arrays.alive().into_iter().map(Node::Array));
        nodes.extend(lists.hashes.alive().into_iter().map(Node::Hash));
        nodes.extend(lists.sets.alive().into_iter().map(Node::Set));
        nodes.extend(lists.instances.alive().into_iter().map(Node::Instance));
        nodes.extend(lists.exceptions.alive().into_iter().map(Node::Exception));
        nodes.extend(lists.blocks.alive().into_iter().map(Node::Block));
        nodes.extend(lists.methods.alive().into_iter().map(Node::Method));
        nodes.extend(lists.bindings.alive().into_iter().map(Node::Binding));
        nodes
    })
}

/// Free the cycles nothing outside them reaches, answering how many listed
/// objects they held.
pub fn collect() -> usize {
    let mut nodes = Vec::new();
    let mut places: HashMap<usize, usize> = HashMap::new();
    for node in listed_nodes() {
        places.entry(node.address()).or_insert_with(|| {
            nodes.push(node);
            nodes.len() - 1
        });
    }
    // How many references each node gets from the other nodes, and which
    // nodes each one holds.
    let mut internal = vec![0usize; nodes.len()];
    let mut edges: Vec<Vec<usize>> = Vec::new();
    let mut unreadable = Vec::new();
    let mut at = 0;
    while at < nodes.len() {
        let mut found = Vec::new();
        if !nodes[at].held(&mut found) {
            unreadable.push(at);
        }
        let mut targets = Vec::with_capacity(found.len());
        for node in found {
            let target = *places.entry(node.address()).or_insert_with(|| {
                nodes.push(node);
                internal.push(0);
                nodes.len() - 1
            });
            internal[target] += 1;
            targets.push(target);
        }
        edges.push(targets);
        at += 1;
    }
    // A node with more references than the nodes and the collector's own
    // copy account for is held from outside, and so is everything it holds.
    let mut reached = vec![false; nodes.len()];
    let mut waiting: Vec<usize> = (0..nodes.len())
        .filter(|&at| nodes[at].count() > internal[at] + 1)
        .chain(unreadable)
        .collect();
    for &at in &waiting {
        reached[at] = true;
    }
    while let Some(at) = waiting.pop() {
        for &target in &edges[at] {
            if !reached[target] {
                reached[target] = true;
                waiting.push(target);
            }
        }
    }
    let mut dropped = Vec::new();
    let mut dropped_variables = Vec::new();
    let mut freed = 0;
    for (node, reached) in nodes.iter().zip(&reached) {
        if !reached {
            node.clear(&mut dropped, &mut dropped_variables);
            freed += usize::from(node.is_listed());
        }
    }
    drop(nodes);
    drop(dropped);
    drop(dropped_variables);
    freed
}
