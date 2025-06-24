use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};

use crate::venv::{
    Venv,
    model::Package,
    utils::{get_dist_info_packages, get_python_dir},
};

use super::metadata::METADATA_FEATURES;

pub fn parse_config_file_contents(contents: String) -> String {
    contents
        .lines()
        .find(|l| l.trim_start().starts_with("version"))
        .expect("Could not find version in config")
        .split('=')
        .nth(1)
        .map(|v| v.trim().to_string())
        .unwrap()
}

pub fn parse_metadata(dist_info_path: PathBuf) -> Result<HashMap<String, String>> {
    // TODO: json config file in the future
    let metadata_path = dist_info_path.join("METADATA");
    let metadata_contents = fs::read_to_string(&metadata_path)
        .with_context(|| {
            format!(
                "Failed to read metadata file at {}",
                metadata_path.display()
            )
        })?
        .replace("\r\n", "\n");
    // WARN: hack above is for linux. needs a cross-platform solution

    let mut header = Vec::new();

    for line in metadata_contents.lines() {
        if line.trim().is_empty() {
            break;
        }
        header.push(line);
    }

    let metadata_values: HashMap<String, String> = header
        .iter()
        .filter_map(|line| line.split_once(": "))
        .filter(|(name, _)| METADATA_FEATURES.contains(name))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();

    Ok(metadata_values)
}
// expects dir to be a virtual environment
pub fn parse_from_dir(dir: &Path) -> Result<Venv> {
    if !dir.is_dir() {
        Err(anyhow!("{} is not directory.", dir.display()))
    } else {
        // println!("Reading dir: {}", dir.to_str().unwrap());
        let pyvevnv_cfg_file = dir.join("pyvenv.cfg");

        let cfg_contents = fs::read_to_string(&pyvevnv_cfg_file)
            .with_context(|| format!("Failed to read {}", pyvevnv_cfg_file.display()))?;

        let _version = parse_config_file_contents(cfg_contents);
        // println!("Python version: {version}");

        #[cfg(target_os = "windows")]
        let binaries = dir.join("Scripts");

        #[cfg(target_os = "linux")]
        let _binaries = dir.join("bin");

        // println!("Binary directory: {}", binaries.to_str().unwrap());

        let lib_dir = dir.join("lib");

        let python_dir = get_python_dir(lib_dir)?
            .context("Could not find python version directory under '/lib'")?;
        let site_packages = python_dir.join("site-packages");
        let dist_info_packages = get_dist_info_packages(site_packages)
            .context("Could not read 'dist-info' directories")?;

        if dist_info_packages.is_empty() {
            return Err(anyhow!("No dist-info packages found in the venv"));
        }

        let mut packages: Vec<Package> = Vec::new();

        for p in &dist_info_packages {
            let metadata_map = parse_metadata(p.to_path_buf())
                .with_context(|| format!("Failed to parse metadata at {}", p.display()))?;

            let package = Package::new(
                metadata_map
                    .get("Name")
                    .context("Missing 'Name' field in METADATA")?,
                metadata_map
                    .get("Version")
                    .context("Missing 'Version' field in METADATA")?,
                metadata_map.clone(),
            );

            packages.push(package);
        }

        let v = Venv::new(dir.file_stem().unwrap().to_str().unwrap(), packages);
        Ok(v)
    }
}
