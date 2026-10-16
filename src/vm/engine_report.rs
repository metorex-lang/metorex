//! What `METOREX_ENGINE_REPORT=path` writes at exit: how many units ran on
//! bytecode and how many on the tree walker, and which node types sent each
//! one there. When the path is a directory, each process writes a file of its
//! own there, named for its process id, which `scripts/engine_report_summary.sh`
//! adds up.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::vm::units::Compiled;

#[derive(Default)]
struct Report {
    on_bytecode: u64,
    on_tree_walker: u64,
    fallbacks: BTreeMap<&'static str, u64>,
}

static REPORT: Mutex<Option<Report>> = Mutex::new(None);
static PATH: Mutex<Option<String>> = Mutex::new(None);

/// Record a compiled unit, when a report was asked for.
pub fn record(compiled: &Compiled) {
    let mut held = REPORT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(report) = held.as_mut() else {
        return;
    };
    match compiled {
        Compiled::Unit(_) => report.on_bytecode += 1,
        Compiled::Fallback(node) => {
            report.on_tree_walker += 1;
            *report.fallbacks.entry(node).or_default() += 1;
        }
    }
}

/// Write the report to the path `METOREX_ENGINE_REPORT` names when the
/// process exits.
pub fn enable_from_environment() {
    let Ok(path) = std::env::var("METOREX_ENGINE_REPORT") else {
        return;
    };
    if path.is_empty() {
        return;
    }
    *PATH.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(path);
    *REPORT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Report::default());
    extern "C" fn write_at_exit() {
        write_report();
    }
    // SAFETY: the handler is a plain function that lives as long as the
    // process does.
    unsafe {
        libc::atexit(write_at_exit);
    }
}

fn write_report() {
    let Some(path) = PATH
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
    else {
        return;
    };
    let held = REPORT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(report) = held.as_ref() else {
        return;
    };
    let mut written = format!(
        "units on bytecode: {}\nunits on the tree walker: {}\nfallbacks:\n",
        report.on_bytecode, report.on_tree_walker
    );
    let mut by_count: Vec<(&&'static str, &u64)> = report.fallbacks.iter().collect();
    by_count.sort_by(|left, right| right.1.cmp(left.1).then(left.0.cmp(right.0)));
    for (node, count) in by_count {
        written.push_str(&format!("  {count} {node}\n"));
    }
    let mut path = std::path::PathBuf::from(path);
    if path.is_dir() {
        path.push(format!("metorex-{}.txt", std::process::id()));
    }
    if let Err(error) = std::fs::write(&path, written) {
        eprintln!(
            "metorex: cannot write the engine report to {}: {error}",
            path.display()
        );
    }
}
