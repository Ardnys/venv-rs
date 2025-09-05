use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    eyre::{self, Result, WrapErr},
    owo_colors::OwoColorize,
};
use venv_rs::dir_size::{self, Chonk};

use crate::venv::{
    Venv,
    metadata::{Metadata, MetadataBuilder, MetadataTokens},
    model::Package,
    utils::get_python_dir,
};

use super::utils::{get_packages, package_pairs};

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

    // TODO: return the builder because we are not done with dependencies yet
    Ok(metadata)
}
// expects dir to be a virtual environment
pub fn parse_from_dir(dir: &Path) -> Result<Venv> {
    let dir = dunce::canonicalize(dir)?;
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

        let lib_dir = if is_windows {
            dir.join("Lib")
        } else {
            dir.join("lib")
        };

        let site_packages = if is_windows {
            lib_dir.join("site-packages")
        } else {
            let python_dir = get_python_dir(lib_dir)?.ok_or_else(|| {
                eyre::eyre!("Could not find python version directory under '/lib'")
            })?;
            python_dir.join("site-packages")
        };

        // println!("{}", site_packages.to_string_lossy());
        let (dist_info_packages, package_dirs) =
            get_packages(site_packages).wrap_err("Could not read 'dist-info' directories")?;

        if dist_info_packages.is_empty() {
            return Err(eyre::eyre!("No dist-info packages found in the venv"));
        }

        let pairs = package_pairs(dist_info_packages, package_dirs);
        let (packages, num_pkg) =
            parse_package_pairs(pairs).context("Error while parsing pairs")?;

        let venv_size = dir_size::ParallelReader
            .get_dir_size(&dir)
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

fn parse_package_pairs(
    pairs: Vec<(Option<PathBuf>, Option<PathBuf>)>,
) -> Result<(Vec<Package>, i32)> {
    let mut packages: Vec<Package> = Vec::new();
    let mut num_pkg = 0;

    for (pkg, dist_info) in &pairs {
        // println!("Pkg: {}", pkg.to_str().unwrap());
        let metadata = if let Some(d) = get_metadata(dist_info) {
            num_pkg += 1;
            d
        } else {
            continue;
        };

        let package_size = get_package_size(pkg);

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
    // TODO: after all packages are done, go through them again and insert their dependencies
    Ok((packages, num_pkg))
}

fn get_metadata(dist_info: &Option<PathBuf>) -> Option<Metadata> {
    if let Some(d) = dist_info {
        match parse_metadata(d.to_path_buf())
            .with_context(|| format!("Failed to parse metadata at {}", d.display()))
        {
            Ok(m) => Some(m),
            Err(err) => {
                eprintln!(
                    "{} {}: {:#}",
                    "error parsing metadata for".red().bold(),
                    d.display().yellow(),
                    err.red().italic()
                );
                Some(MetadataBuilder::default().build())
            }
        }
    } else {
        None
    }
}

fn get_package_size(pkg: &Option<PathBuf>) -> u64 {
    if let Some(p) = pkg {
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
    }
}

// fn insert_dependencies(packages: &Vec<Rc<Package>>) {}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::tempdir;

    use super::*;
    use std::{collections::HashSet, fs::File};

      #[test]
    fn test_parse_metadata() -> Result<()> {
        // 1. Create a temporary directory.
        let dir = tempdir().unwrap();
        let dist_info_path = dir.path().join("my_package-1.0.dist-info");
        fs::create_dir(&dist_info_path).unwrap();

        // 2. Create the METADATA file with known content.
        let metadata_path = dist_info_path.join("METADATA");
        let mut file = File::create(metadata_path).unwrap();
        let mock_metadata_contents = "
Metadata-Version: 2.4
Name: pillow
Version: 11.3.0
Summary: Python Imaging Library (Fork)
License-Expression: MIT-CMU
Requires-Python: >=3.9
Description-Content-Type: text/markdown
License-File: LICENSE
Provides-Extra: docs
Requires-Dist: furo; extra == \"docs\"
Requires-Dist: olefile; extra == \"docs\"
Requires-Dist: sphinx>=8.2; extra == \"docs\"
Requires-Dist: sphinx-autobuild; extra == \"docs\"
Requires-Dist: sphinx-copybutton; extra == \"docs\"
Requires-Dist: sphinx-inline-tabs; extra == \"docs\"
Requires-Dist: sphinxext-opengraph; extra == \"docs\"
Provides-Extra: fpx
Requires-Dist: olefile; extra == \"fpx\"
Provides-Extra: mic
Requires-Dist: olefile; extra == \"mic\"
Provides-Extra: test-arrow
Requires-Dist: pyarrow; extra == \"test-arrow\"
Provides-Extra: tests
Requires-Dist: check-manifest; extra == \"tests\"
Requires-Dist: coverage>=7.4.2; extra == \"tests\"
Requires-Dist: defusedxml; extra == \"tests\"
Requires-Dist: markdown2; extra == \"tests\"
Requires-Dist: olefile; extra == \"tests\"
Requires-Dist: packaging; extra == \"tests\"
Requires-Dist: pyroma; extra == \"tests\"
Requires-Dist: pytest; extra == \"tests\"
Requires-Dist: pytest-cov; extra == \"tests\"
Requires-Dist: pytest-timeout; extra == \"tests\"
Requires-Dist: pytest-xdist; extra == \"tests\"
Requires-Dist: trove-classifiers>=2024.10.12; extra == \"tests\"
Provides-Extra: typing
Requires-Dist: typing-extensions; python_version < \"3.10\" and extra == \"typing\"
Provides-Extra: xmp
Requires-Dist: defusedxml; extra == \"xmp\"
Dynamic: license-file"
            .trim();

        write!(file, "{mock_metadata_contents}\n\n").unwrap();

        // 3. Call the function with the path to the temp directory.
        let metadata_result = parse_metadata(dist_info_path);
        assert!(metadata_result.is_ok());
        let metadata = metadata_result.unwrap();

        // 4. Assert the returned metadata is correct.
        assert_eq!(metadata.name, "pillow");
        assert_eq!(metadata.version, "11.3.0");
        assert_eq!(metadata.summary, "Python Imaging Library (Fork)");
        assert!(metadata.dependencies.is_some());
        // TODO: this dependency thing is more complicated
        assert!(metadata.dependencies.unwrap().contains("pyarrow"));
        Ok(())
    }

    #[test]
    fn test_parse_metadata_file_not_found() {
        let dir = tempdir().unwrap();
        let non_existent_path = dir.path().join("non_existent.dist-info");
        let result = parse_metadata(non_existent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_from_dir() {
        // This would be a larger integration test.
        // 1. Create a full mock virtual environment in a temp directory.
        let venv_dir = tempdir().unwrap();

        // Create pyvenv.cfg
        let mut pyvenv_cfg = File::create(venv_dir.path().join("pyvenv.cfg")).unwrap();
        // TODO: windows versions as well
        let pyvenv_cfg_contents = "
home = /usr/local/bin
include-system-site-packages = false
version = 3.13.2
executable = /usr/local/bin/python3.13
command = /usr/local/bin/python3 -m venv /home/user/projects/python/imgs/fdmp
"
        .trim();

        write!(pyvenv_cfg, "{pyvenv_cfg_contents}").unwrap();

        // Create directory structure
        let is_windows = cfg!(windows);
        let bin_dir = if is_windows {
            venv_dir.path().join("Scripts")
        } else {
            venv_dir.path().join("bin")
        };
        let lib_dir = if is_windows {
            venv_dir.path().join("Lib")
        } else {
            venv_dir.path().join("lib")
        };

        let site_packages = if is_windows {
            lib_dir.join("site-packages")
        } else {
            let python_dir = lib_dir.join("python3.13");
            python_dir.join("site-packages")
        };
        fs::create_dir_all(&site_packages).unwrap();

        // Create a package (pip is automatically generated for example)
        let package_dir = site_packages.join("pip");
        fs::create_dir(&package_dir).unwrap();
        let mut dummy_file = File::create(package_dir.join("__init__.py")).unwrap();
        writeln!(dummy_file, "print('hello from pip')").unwrap();

        // Create its dist-info
        let dist_info_dir = site_packages.join("pip-25.1.1.dist-info");
        fs::create_dir(&dist_info_dir).unwrap();
        let mut metadata_file = File::create(dist_info_dir.join("METADATA")).unwrap();

        let pip_metadata = "
Metadata-Version: 2.4
Name: pip
Version: 25.1.1
Summary: The PyPA recommended tool for installing Python packages.
License: MIT
Requires-Python: >=3.9
Description-Content-Type: text/x-rst
License-File: LICENSE.txt
License-File: AUTHORS.txt
Dynamic: license-file
"
        .trim();

        write!(metadata_file, "{pip_metadata}\n\n").unwrap();

        // 2. Call parse_from_dir
        let venv_result = parse_from_dir(venv_dir.path());
        assert!(venv_result.is_ok());
        let venv = venv_result.unwrap();

        // 3. Assert properties of the Venv struct
        assert_eq!(
            venv.name,
            venv_dir.path().file_stem().unwrap().to_str().unwrap()
        );
        assert_eq!(venv.version, "3.13.2");
        assert_eq!(venv.num_dist_info_packages, 1);
        assert_eq!(venv.packages[0].name, "pip");
        assert_eq!(venv.packages[0].version, "25.1.1");
        assert_eq!(venv.binaries, bin_dir);
        assert_eq!(venv.path, venv_dir.path().to_path_buf());
    }

    #[test]
    fn test_metadata_builder_build() {
        let mut builder = MetadataBuilder::new();
        let mut deps = HashSet::new();
        deps.insert("numpy".to_string());
        deps.insert("pillow".to_string());
        deps.insert("opencv-python".to_string());

        let metadata = builder
            .name("fimage".to_string())
            .version("0.2.1".to_string())
            .summary("A Python module to create and apply filters to images.".to_string())
            .add_dependencies(deps.clone())
            .build();

        assert_eq!(metadata.name, "fimage");
        assert_eq!(metadata.version, "0.2.1");
        assert_eq!(
            metadata.summary,
            "A Python module to create and apply filters to images."
        );
        assert_eq!(metadata.dependencies, Some(deps));
    }

    #[test]
    fn test_metadata_builder_build_default() {
        let mut builder = MetadataBuilder::default();
        let metadata = builder.build();
        assert_eq!(metadata.name, "");
        assert_eq!(metadata.version, "");
        assert_eq!(metadata.summary, "");
        assert!(metadata.dependencies.is_none());
    }
}
