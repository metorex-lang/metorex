//! The core library and the libraries a default start loads, parsed when
//! metorex was built. Each copy holds its statements and the symbol names
//! parsing it recorded.

use crate::ast::Statement;
use crate::vm::stdlib::source_digest::source_digest;

// Defines `PRELUDE`, the core library's copy, and `LIBRARIES`, each
// library's source digest, source length, and copy.
include!(concat!(env!("OUT_DIR"), "/parsed_sources.rs"));

/// The core library's statements.
pub(crate) fn prelude() -> Vec<Statement> {
    decode(PRELUDE)
}

/// The statements of a library the binary holds parsed, when `source` is
/// one of them.
pub(crate) fn library(source: &str) -> Option<Vec<Statement>> {
    let candidates: Vec<_> = LIBRARIES
        .iter()
        .filter(|(_, length, _)| *length == source.len())
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let digest = source_digest(source);
    candidates
        .into_iter()
        .find(|(held, _, _)| *held == digest)
        .map(|(_, _, parsed)| decode(parsed))
}

fn decode(parsed: &[u8]) -> Vec<Statement> {
    let _counted =
        crate::vm::memory_report::attribute(crate::vm::memory_report::Category::SyntaxTree);
    let (statements, symbols): (Vec<Statement>, Vec<String>) =
        rmp_serde::from_slice(parsed).expect("the build script wrote this copy");
    for name in &symbols {
        crate::symbol_registry::record(name);
    }
    statements
}
