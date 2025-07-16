use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    eyre::{self, Result, WrapErr},
    owo_colors::OwoColorize,
};
use venv_rs::dir_size::{self, Chonk};

use crate::venv::{Venv, model::Package, utils::get_python_dir};

use super::utils::{get_packages, package_pairs};

pub enum MetadataTokens {
    Name(String),
    Version(String),
    Summary(String),
    Dependency(String),
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub depedencies: Option<HashSet<String>>,
}

#[derive(Default)]
pub struct MetadataBuilder {
    pub name: Option<String>,
    pub version: Option<String>,
    pub summary: Option<String>,
    pub depedencies: Option<HashSet<String>>,
}

impl MetadataBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            summary: None,
            depedencies: None,
        }
    }
    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }
    pub fn version(&mut self, version: String) -> &mut Self {
        self.version = Some(version);
        self
    }
    pub fn summary(&mut self, summary: String) -> &mut Self {
        self.summary = Some(summary);
        self
    }
    pub fn add_dependencies(&mut self, dependencies: HashSet<String>) -> &mut Self {
        self.depedencies = Some(dependencies);
        self
    }
    pub fn build(&mut self) -> Metadata {
        Metadata {
            name: self.name.clone().unwrap_or_default(),
            version: self.version.clone().unwrap_or_default(),
            summary: self.summary.clone().unwrap_or_default(),
            depedencies: self.depedencies.clone(),
        }
    }
}

impl Metadata {
    pub fn parse_tokens(tokens: Vec<MetadataTokens>) -> color_eyre::Result<Metadata> {
        let mut builder = &mut MetadataBuilder::default();
        let mut dependencies: HashSet<String> = HashSet::new();
        for tok in tokens {
            match tok {
                MetadataTokens::Name(name) => builder = builder.name(name),
                MetadataTokens::Version(version) => builder = builder.version(version),
                MetadataTokens::Summary(summary) => builder = builder.summary(summary),
                MetadataTokens::Dependency(dep) => {
                    let dep_name = Self::parse_dependency(dep);
                    let _ = dependencies.insert(dep_name);
                }
            }
        }
        if !dependencies.is_empty() {
            builder.add_dependencies(dependencies);
        }
        let md = builder.build();
        Ok(md)
    }

    fn parse_dependency(dep: String) -> String {
        Self::split_at_separator(dep)
    }

    fn split_at_separator(dep: String) -> String {
        let mut split_index = 0;
        for (i, c) in dep.char_indices() {
            match c {
                '>' | '=' | ' ' | '\n' | ';' | '!' | '<' => {
                    split_index = i;
                    break;
                }
                _ => { /* consume */ }
            }
        }
        if split_index == 0 {
            split_index = dep.len();
        }

        dep.split_at(split_index).0.to_string()
    }
}

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

pub fn parse_metadata(dist_info_path: PathBuf) -> Result<Metadata> {
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

    let tokens: Vec<MetadataTokens> = header
        .iter()
        .filter_map(|line| line.split_once(": "))
        .filter_map(|(key, value)| match key {
            "Name" => Some(MetadataTokens::Name(value.to_string())),
            "Version" => Some(MetadataTokens::Version(value.to_string())),
            "Summary" => Some(MetadataTokens::Summary(value.to_string())),
            "Requires-Dist" => Some(MetadataTokens::Dependency(value.to_string())),
            _ => None, // skip unknown keys
        })
        .collect();

    let metadata = Metadata::parse_tokens(tokens)?;

    Ok(metadata)
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

        let is_windows = cfg!(windows);

        let binaries = if is_windows {
            dir.join("Scripts")
        } else {
            dir.join("bin")
        };

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
        let mut num_pkg = 0;

        for (pkg, dist_info) in &pairs {
            // println!("Pkg: {}", pkg.to_str().unwrap());
            let metadata = if let Some(d) = dist_info {
                match parse_metadata(d.to_path_buf())
                    .with_context(|| format!("Failed to parse metadata at {}", d.display()))
                {
                    Ok(m) => {
                        num_pkg += 1;
                        m
                    }
                    Err(err) => {
                        eprintln!(
                            "{} {}: {:#}",
                            "error parsing metadata for".red().bold(),
                            d.display().yellow(),
                            err.red().italic()
                        );
                        MetadataBuilder::default().build()
                    }
                }
            } else {
                continue;
                // MetadataBuilder::default().build()
            };

            let package_size = if let Some(p) = pkg {
                match dir_size::ParallelReader.get_dir_size(p) {
                    Ok(sz) => sz,
                    Err(err) => {
                        eprintln!(
                            "{} {}: {:#}",
                            "Error while calculating size: ".red().bold(),
                            p.display().yellow(),
                            err.red().italic()
                        );
                        0
                    }
                }
            } else {
                0
            };

            let dist_info_size = if let Some(d) = dist_info {
                dir_size::ParallelReader
                    .get_dir_size(d)
                    .context("Could not get dist-info size")?
            } else {
                0
            };

            let package = Package::new(
                &metadata.name,
                &metadata.version,
                package_size + dist_info_size,
                metadata.clone(),
            );
            // println!("pck: {:?}", package);

            packages.push(package);
        }

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
            dir.to_path_buf(),
        );
        Ok(v)
    }
}
