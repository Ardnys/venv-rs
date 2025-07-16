use std::path::PathBuf;

use walkdir::WalkDir;

pub fn search_venvs(path: PathBuf) -> Vec<PathBuf> {
    let mut venv_paths = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str().unwrap_or_default() == "pyvenv.cfg" {
            venv_paths.push(entry.path().parent().unwrap().to_path_buf());
        }
    }
    venv_paths
}
