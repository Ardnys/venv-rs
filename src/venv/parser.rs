use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    eyre::{self, Result, WrapErr},
    owo_colors::OwoColorize,
};
use venv_rs::dir_size::{self, Chonk};

use crate::venv::{Venv, model::Package, utils::get_python_dir};

use super::{
    metadata::METADATA_FEATURES,
    utils::{get_packages, package_pairs},
};

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
        Err(eyre::eyre!("{} is not directory.", dir.display()))
    } else {
        // println!("Reading dir: {}", dir.to_str().unwrap());
        let pyvevnv_cfg_file = dir.join("pyvenv.cfg");

        let cfg_contents = fs::read_to_string(&pyvevnv_cfg_file)
            .with_context(|| format!("Failed to read {}", pyvevnv_cfg_file.display()))?;

        let version = parse_config_file_contents(cfg_contents);
        // println!("Python version: {version}");

        #[cfg(target_os = "windows")]
        let binaries = dir.join("Scripts");

        #[cfg(target_os = "linux")]
        let binaries = dir.join("bin");

        // println!("Binary directory: {}", binaries.to_str().unwrap());

        let lib_dir = dir.join("lib");

        let python_dir = get_python_dir(lib_dir)?
            .ok_or_else(|| eyre::eyre!("Could not find python version directory under '/lib'"))?;
        let site_packages = python_dir.join("site-packages");
        let (dist_info_packages, package_dirs) =
            get_packages(site_packages).wrap_err("Could not read 'dist-info' directories")?;

        if dist_info_packages.is_empty() {
            return Err(eyre::eyre!("No dist-info packages found in the venv"));
        }

        let pairs = package_pairs(dist_info_packages, package_dirs);

        let mut packages: Vec<Package> = Vec::new();

        for (pkg, dist_info) in &pairs {
            let metadata_map = if let Some(d) = dist_info {
                match parse_metadata(d.to_path_buf())
                    .with_context(|| format!("Failed to parse metadata at {}", d.display()))
                {
                    Ok(m) => m,
                    Err(err) => {
                        eprintln!(
                            "{} {}: {:#}",
                            "error parsing metadata for".red().bold(),
                            d.display().yellow(),
                            err.red().italic()
                        );
                        HashMap::new()
                    }
                }
            } else {
                HashMap::new()
            };

            let package_size = match dir_size::ParallelReader.get_dir_size(pkg) {
                Ok(sz) => sz,
                Err(err) => {
                    eprintln!(
                        "{} {}: {:#}",
                        "Error while calculating size: ".red().bold(),
                        pkg.display().yellow(),
                        err.red().italic()
                    );
                    0
                }
            };

            let dist_info_size = if let Some(d) = dist_info {
                dir_size::ParallelReader
                    .get_dir_size(d)
                    .context("Could not get dist-info size")?
            } else {
                0
            };

            let package = Package::new(
                metadata_map
                    .get("Name")
                    .map_or(pkg.file_stem().unwrap().to_str().unwrap_or(""), |n| n),
                metadata_map.get("Version").map_or("NIL", |n| n),
                package_size + dist_info_size,
                metadata_map.clone(),
            );

            packages.push(package);
        }

        let num_pkg = packages
            .iter()
            .filter(|&x| !x.metadata.is_empty())
            .fold(0, |acc, _| acc + 1);

        let venv_size = dir_size::ParallelReader
            .get_dir_size(dir)
            .context("Could not get venv size")?;

        let v = Venv::new(
            dir.file_stem().unwrap().to_str().unwrap(),
            version,
            venv_size,
            packages,
            num_pkg,
            binaries,
        );
        Ok(v)
    }
}
