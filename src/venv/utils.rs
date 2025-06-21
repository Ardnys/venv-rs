use std::{fs, io, path::PathBuf};

use anyhow::Result;

pub fn get_python_dir(lib_dir: PathBuf) -> io::Result<Option<PathBuf>> {
    let mut entries = fs::read_dir(lib_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir());

    Ok(entries.next().map(|e| e.path()))
}

pub fn get_dist_info_packages(site_packages: PathBuf) -> Result<Vec<PathBuf>> {
    let dist_info_dirs: Vec<PathBuf> = fs::read_dir(site_packages)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with("dist-info"))
                .unwrap_or(false)
        })
        .collect();

    Ok(dist_info_dirs)
}

// TODO: implement this function
fn get_package_size(package_dir: PathBuf) -> f32 {
    todo!()
}
