//! The path the running binary was started through. A binary started through
//! a link named `ruby` answers that link, as an installed `ruby` would, where
//! `std::env::current_exe` answers the link's target on Linux.

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

/// The path the binary was started through, found once, before a program can
/// change directory or write over its first argument by assigning `$0`.
pub fn interpreter_path() -> Option<&'static Path> {
    static STARTED_THROUGH: OnceLock<Option<PathBuf>> = OnceLock::new();
    STARTED_THROUGH
        .get_or_init(|| {
            std::env::args_os()
                .next()
                .and_then(|name| started_through(&name))
                .or_else(|| std::env::current_exe().ok())
        })
        .as_deref()
}

/// The file the first argument names: a path with a slash in it, taken from
/// the working directory when relative, or a bare name looked up on `PATH`.
fn started_through(name: &OsStr) -> Option<PathBuf> {
    let named = Path::new(name);
    if named.components().count() > 1 {
        let joined = std::env::current_dir().ok()?.join(named);
        let path: PathBuf = joined
            .components()
            .filter(|component| *component != Component::CurDir)
            .collect();
        return is_executable(&path).then_some(path);
    }
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|directory| directory.join(named))
        .find(|path| is_executable(path))
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
