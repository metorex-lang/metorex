// The file running now, and the files already loaded.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The path `__FILE__` reports for the file running now: the spelling the
    /// main script was named by, and the current file for anything else.
    /// The file the code running now was written in, which a frame
    /// records as the file its call site is in.
    pub(crate) fn file_for_frames(&self) -> Option<String> {
        self.current_source_file.clone().or_else(|| {
            self.reported_current_file()
                .map(|file| file.display().to_string())
        })
    }

    /// `file_for_frames` as a shared string, made once while the file
    /// stays the same.
    pub(crate) fn frames_file(&mut self) -> Option<Rc<str>> {
        if let Some(current) = &self.current_source_file
            && let Some((text, shared)) = &self.frames_file_held
            && text == current
        {
            return Some(Rc::clone(shared));
        }
        let text = self.file_for_frames()?;
        let shared: Rc<str> = Rc::from(text.as_str());
        if self.current_source_file.as_ref() == Some(&text) {
            self.frames_file_held = Some((text, Rc::clone(&shared)));
        }
        Some(shared)
    }

    pub(crate) fn reported_current_file(&self) -> Option<PathBuf> {
        let current = self.current_file.as_ref()?;
        Some(self.reported_spelling(current))
    }

    /// The path `__FILE__` reports for `current`.
    pub(crate) fn reported_spelling(&self, current: &PathBuf) -> PathBuf {
        match &self.script_path {
            Some((canonical, as_given)) if canonical == current => as_given.clone(),
            _ => self
                .reported_files
                .get(current)
                .cloned()
                .unwrap_or_else(|| current.clone()),
        }
    }

    /// Get the current file being executed.
    pub fn get_current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    /// Mark a file as loaded in the registry.
    pub fn mark_file_loaded(&mut self, path: PathBuf) {
        self.loaded_files.insert(path);
    }

    /// Check if a file has already been loaded.
    pub fn is_file_loaded(&self, path: &PathBuf) -> bool {
        self.loaded_files.contains(path)
    }

    /// Drop the loaded-files entry for `path`. Used by autoload's error path
    /// so a subsequent constant access re-runs the file rather than skipping
    /// it via deduplication. Also drops a matching string from
    /// `$LOADED_FEATURES` so `autoload?` / `defined?` don't observe a stale
    /// "loaded" status after a failed load.
    pub fn unmark_file_loaded(&mut self, path: &PathBuf) {
        self.loaded_files.remove(path);
        let path_str = path.to_string_lossy().into_owned();
        if let Some(View::Array(arr)) = self.globals().get("\"").view() {
            arr.borrow_mut()
                .retain(|o| !matches!(o.view(), View::String(s) if *s.as_str() == *path_str));
        }
    }

    /// Prepend a path to the `$LOAD_PATH` (`$:`) global array.
    pub fn prepend_load_path(&mut self, path: String) {
        if let Some(View::Array(arr)) = self.globals.get(":").view() {
            arr.borrow_mut().insert(0, Object::string(path));
        }
    }

    /// Add a directory to the end of `$LOAD_PATH`, where the directories a
    /// library is installed into sit.
    pub fn append_load_path(&mut self, path: String) {
        if let Some(View::Array(arr)) = self.globals.get(":").view() {
            arr.borrow_mut().push(Object::string(path));
        }
    }

    /// Add one of the directories metorex's own libraries are installed in.
    /// Ruby marks each of these with `@gem_prelude_index`, naming itself,
    /// which is what tells them apart from a directory the program added.
    pub fn append_installed_load_path(&mut self, path: String) {
        let entry = Object::string(path.clone());
        if let View::String(held) = entry.view() {
            held.freeze();
        }
        if let Some(address) = Self::collection_address(&entry) {
            self.collection_variables
                .entry(address)
                .or_default()
                .insert("gem_prelude_index".to_string(), Object::string(path));
            self.collection_variable_owners
                .insert(address, entry.clone());
        }
        if let Some(View::Array(arr)) = self.globals.get(":").view() {
            arr.borrow_mut().push(entry);
        }
    }

    /// Load a library `-r` named, which runs before the script does. Its
    /// top-level locals are its own rather than the script's, so a name it
    /// binds is not one the script or TOPLEVEL_BINDING goes on to see.
    pub fn require_startup_library(&mut self, name: &str) -> Result<(), MetorexError> {
        self.environment_mut().push_isolated_scope();
        // MRI loads these through RubyGems, and lists them under paths of
        // their own, so they are left out of `$LOADED_FEATURES` here.
        let listing = std::mem::replace(&mut self.lists_carried_features, false);
        let answer = self.require_library(name);
        self.lists_carried_features = listing;
        self.environment_mut().pop_scope();
        answer
    }

    pub fn require_library(&mut self, name: &str) -> Result<(), MetorexError> {
        let expanded = self.expand_home_path(name);
        let name = expanded.as_str();
        let search_dirs = self.load_path_directories();

        // An absolute path, or one written relative to the working directory,
        // names the file outright rather than being searched for.
        let mut found_path = None;
        let direct = std::path::PathBuf::from(name);
        if direct.is_absolute() || name.starts_with("./") || name.starts_with("../") {
            for candidate in [direct.clone(), direct.with_extension("rb")] {
                if candidate.is_file() {
                    found_path = Some(candidate);
                    break;
                }
            }
        }
        for dir in &search_dirs {
            if found_path.is_some() {
                break;
            }
            let base = std::path::PathBuf::from(dir);
            // Try `.rb` first so a matching .rb file wins over a sibling directory.
            let candidates = [base.join(format!("{}.rb", name)), base.join(name)];
            for candidate in &candidates {
                if candidate.is_file() {
                    found_path = Some(candidate.clone());
                    break;
                }
            }
            if found_path.is_some() {
                break;
            }
        }

        // A library metorex carries is used when the load path holds no file
        // of that name.
        if found_path.is_none()
            && let Some(source) = crate::vm::stdlib::embedded_library(name)
        {
            self.run_embedded_library(name, source)?;
            return Ok(());
        }

        let resolved = found_path.ok_or_else(|| {
            MetorexError::runtime_error(
                format!(
                    "cannot load such file -- {} (searched in $LOAD_PATH: {:?})",
                    name, search_dirs
                ),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        self.execute_file(&resolved).map_err(|e| {
            keep_exception(e, |message| {
                MetorexError::runtime_error(
                    format!("require('{}') — {}", name, message),
                    SourceLocation::new(0, 0, 0),
                )
            })
        })?;

        Ok(())
    }

    /// Execute a file with automatic deduplication and path tracking.
    ///
    /// This method loads and executes a file, handling:
    /// - File deduplication (files are only executed once)
    /// - Current file path tracking (for require_relative)
    /// - Automatic path canonicalization
    /// - Proper restoration of the previous current file
    pub fn execute_file(&mut self, path: &std::path::Path) -> Result<Object, MetorexError> {
        self.execute_file_recording(path, true)
    }
}
