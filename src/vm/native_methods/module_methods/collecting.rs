// What the collector accounts for.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// How many collections have run.
    pub(crate) fn gc_collection_count(&self) -> i64 {
        match self.globals().get("__gc_count").view() {
            Some(View::Int(count)) => count,
            _ => 0,
        }
    }

    /// The nanoseconds spent collecting.
    pub(crate) fn gc_total_time(&self) -> i64 {
        match self.globals().get("__gc_total_time").view() {
            Some(View::Int(nanoseconds)) => nanoseconds,
            _ => 0,
        }
    }

    /// Record that a collection ran and how long it took.
    pub(crate) fn record_gc_run(&mut self, spent: std::time::Duration) {
        let count = self.gc_collection_count() + 1;
        let total = self.gc_total_time() + (spent.as_nanos() as i64).max(1);
        self.globals_mut().set("__gc_count", Object::Int(count));
        self.globals_mut()
            .set("__gc_total_time", Object::Int(total));
    }

    /// Run the cycle collector unless the program switched it off, clear the
    /// dead out of the list of objects, and run the finalizers of the
    /// objects freed since the last time.
    pub(crate) fn collect_garbage(&mut self, position: Position) -> Result<(), MetorexError> {
        if crate::object::live::collecting() {
            let started = std::time::Instant::now();
            self.collect_unreached_data(position)?;
            crate::object::collector::collect();
            self.record_gc_run(started.elapsed());
        }
        crate::object::live::settle();
        self.run_freed_finalizers(position)
    }

    /// Run the finalizers of each object freed since the last time this was
    /// asked, in the order they were defined.
    pub(crate) fn run_freed_finalizers(&mut self, position: Position) -> Result<(), MetorexError> {
        if self.finalized_objects.is_empty() {
            return Ok(());
        }
        let (alive, freed): (Vec<_>, Vec<_>) = std::mem::take(&mut self.finalized_objects)
            .into_iter()
            .partition(|(target, _)| target.alive());
        self.finalized_objects = alive;
        if freed.is_empty() {
            return Ok(());
        }
        let Some(object_space) = self.globals().get("ObjectSpace") else {
            return Ok(());
        };
        for (_, id) in freed {
            self.send_to_object(object_space.clone(), "__finalize__", vec![id], position)?;
        }
        Ok(())
    }

    /// The WeakRef handle an argument names, when it names one.
    pub(crate) fn weak_handle(&self, argument: Option<&Object>) -> Option<usize> {
        match argument.view() {
            Some(View::Int(handle)) => usize::try_from(handle)
                .ok()
                .filter(|at| *at < self.weak_references.len()),
            _ => None,
        }
    }

    /// How many objects of each kind the list holds alive, as pairs of the
    /// name MRI gives the kind and the count.
    pub(crate) fn counted_objects(&self) -> Object {
        let counted = crate::object::live::with_lists(|lists| {
            let classes = lists.classes.alive();
            let modules = classes.iter().filter(|one| one.is_module()).count();
            let data = lists.blocks.alive().len()
                + lists.methods.alive().len()
                + lists.bindings.alive().len();
            [
                (
                    "T_OBJECT",
                    lists.instances.alive().len() + lists.exceptions.alive().len(),
                ),
                ("T_CLASS", classes.len() - modules),
                ("T_MODULE", modules),
                ("T_STRING", lists.strings.alive().len()),
                ("T_ARRAY", lists.arrays.alive().len()),
                (
                    "T_HASH",
                    lists.hashes.alive().len() + lists.sets.alive().len(),
                ),
                ("T_DATA", data),
            ]
        });
        Object::array(
            counted
                .into_iter()
                .map(|(kind, count)| {
                    Object::array(vec![Object::symbol(kind), Object::Int(count as i64)])
                })
                .collect(),
        )
    }
}
