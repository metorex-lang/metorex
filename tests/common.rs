// Common test utilities and constants

pub const EXAMPLES_DIR: &str = "tests/_examples";

/// Keep what an example printed under the directory `METOREX_EXAMPLE_OUTPUTS`
/// names, at the example's own path with `.out` added, for the metorex.org
/// examples gallery to show. Does nothing when the variable is unset.
pub fn record_example_output(path: &str, printed: &[u8]) {
    let Some(directory) = std::env::var_os("METOREX_EXAMPLE_OUTPUTS") else {
        return;
    };
    let recorded = std::path::Path::new(&directory).join(format!("{path}.out"));
    if let Some(parent) = recorded.parent() {
        std::fs::create_dir_all(parent).expect("failed to make the output directory");
    }
    std::fs::write(&recorded, printed).expect("failed to record the example's output");
}
