//! Which engine runs the program: the tree walker or bytecode, chosen by
//! `METOREX_ENGINE` once as the process starts.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    Tree,
    Bytecode,
}

static CHOSEN: OnceLock<Engine> = OnceLock::new();

/// The engine the program runs on, the tree walker unless
/// `METOREX_ENGINE` chose bytecode.
pub fn engine() -> Engine {
    CHOSEN.get().copied().unwrap_or(Engine::Tree)
}

/// Read `METOREX_ENGINE`, answering the message to end the run with when it
/// names neither engine.
pub fn choose_from_environment() -> Result<(), String> {
    let chosen = match std::env::var("METOREX_ENGINE") {
        Err(_) => Engine::Tree,
        Ok(named) => match named.as_str() {
            "" | "tree" => Engine::Tree,
            "bytecode" => Engine::Bytecode,
            _ => {
                return Err(format!(
                    "METOREX_ENGINE must be tree or bytecode, not {named:?}"
                ));
            }
        },
    };
    let _ = CHOSEN.set(chosen);
    Ok(())
}
