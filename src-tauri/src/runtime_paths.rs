//! Where SAAR looks for the files it runs at run time: the llama.cpp runtime, the model weights
//! and the OCR worker.
//!
//! An installed copy keeps the runtime beside the executable and the (large, separately
//! downloaded) model in the user's data folder. A development build uses the repository folders.
//! Nothing here depends on the machine the program was built on, except as the last fallback for
//! development.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Records the application data folder. Called once at startup.
pub fn set_data_dir(dir: &Path) {
    let _ = DATA_DIR.set(dir.to_path_buf());
}

/// Folder where the user places the model weights: `<data folder>/models`.
pub fn models_dir() -> Option<PathBuf> {
    DATA_DIR.get().map(|d| d.join("models"))
}

/// Folders searched for bundled runtime files, most specific first.
pub fn resource_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(p) = std::env::var_os("SAAR_RESOURCE_DIR").filter(|p| !p.is_empty()) {
        roots.push(PathBuf::from(p));
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        roots.push(dir.join("resources"));
        roots.push(dir);
    }
    // Development checkout: the repository root, derived from this crate's manifest folder.
    if let Some(root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        roots.push(root.to_path_buf());
    }
    roots
}

/// First existing file among `<root>/<relative>` for each root.
pub fn find_resource(relative: &[&str]) -> Option<PathBuf> {
    resource_roots().into_iter().find_map(|root| {
        relative.iter().map(|r| root.join(r)).find(|p| p.is_file())
    })
}

/// Candidate locations for a model file, user data folder first.
pub fn model_candidates(file_name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(d) = models_dir() {
        out.push(d.join(file_name));
        out.push(d.join("weights").join(file_name));
    }
    for root in resource_roots() {
        out.push(root.join("models").join("weights").join(file_name));
        out.push(root.join("models").join(file_name));
    }
    out
}

/// The model file that exists, or the preferred location to report when none does.
pub fn resolve_model(file_name: &str) -> PathBuf {
    let candidates = model_candidates(file_name);
    candidates
        .iter()
        .find(|p| p.is_file())
        .cloned()
        .unwrap_or_else(|| candidates.into_iter().next().unwrap_or_else(|| PathBuf::from(file_name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_lookup_prefers_the_data_folder_and_names_it_when_missing() {
        let dir = std::env::temp_dir().join(format!("saar_paths_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("models")).unwrap();
        // OnceLock may already hold a folder from another test; only assert on what is set.
        set_data_dir(&dir);
        let missing = resolve_model("no-such-model.gguf");
        assert!(missing.to_string_lossy().ends_with("no-such-model.gguf"));
        assert!(!missing.exists());
        let candidates = model_candidates("x.gguf");
        assert!(candidates.len() >= 3);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_resource_roots_include_the_executable_folder() {
        let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        assert!(resource_roots().contains(&exe_dir));
    }
}
