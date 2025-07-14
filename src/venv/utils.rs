use std::{fs, io, path::PathBuf};

use color_eyre::Result;

pub fn get_python_dir(lib_dir: PathBuf) -> io::Result<Option<PathBuf>> {
    let mut entries = fs::read_dir(lib_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir());

    Ok(entries.next().map(|e| e.path()))
}

pub fn get_dist_info_packages(site_packages: PathBuf) -> Result<Vec<PathBuf>> {
    let (dist_info_dirs, _) = get_packages(site_packages)?;

    Ok(dist_info_dirs)
}

/// For each `pkg` in `packages`, find a matching `.dist-info` directory in
/// `dist_infos` whose name follows the `{name}-{version}.dist-info` pattern
/// and whose `{name}` equals the package’s file name. If none is found, pair
/// with `None`.
pub fn package_pairs(
    dist_infos: Vec<PathBuf>,
    packages: Vec<PathBuf>,
) -> Vec<(Option<PathBuf>, Option<PathBuf>)> {
    // TODO: this ain't working that well anymore
    let mut pairs = Vec::with_capacity(packages.len());

    for pkg in packages.into_iter() {
        let pkg_name = pkg.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let matched = dist_infos.iter().find_map(|di| {
            // dist-info anatomy {name}-{version}.dist-info
            // name anatomy {name} , though with a lot edge cases
            // Case 1. {name}.libs (such as opencv)
            // Case 2. split at _ {name_suffix} -> {name}
            // Case 3. remove _ {name_suffix} -> {namesuffix}
            // Case 4. unpredictable things like scikit-learn imported as sklearn

            let di_fname = di.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let without_suffix = di_fname.strip_suffix(".dist-info").unwrap();

            let (name_part, _) = without_suffix.split_once("-")?;

            if name_part == pkg_name {
                Some(di.clone())
            } else {
                None
            }
        });

        pairs.push((Some(pkg.clone()), matched));
    }

    // at last, check pairs and fill missing dist-info
    for d in &dist_infos {
        if !pairs.iter().any(|(_, di)| *di == Some(d.to_path_buf())) {
            pairs.push((None, Some(d.to_path_buf())));
        }
    }

    pairs
}

pub fn get_packages(site_packages: PathBuf) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (mut dist_info_dirs, mut package_dirs): (Vec<PathBuf>, Vec<PathBuf>) =
        fs::read_dir(site_packages)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .partition(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.ends_with("dist-info"))
                    .unwrap_or(false)
            });

    // WARN: this isn't nice but eh
    dist_info_dirs.sort();
    package_dirs.sort();

    Ok((dist_info_dirs, package_dirs))
}

// my poor function. maybe it worked
pub fn pair_packages(
    dist_infos: Vec<PathBuf>,
    packages: Vec<PathBuf>,
) -> Vec<(PathBuf, Option<PathBuf>)> {
    let mut v: Vec<(PathBuf, Option<PathBuf>)> = Vec::with_capacity(packages.len());
    for p in &packages {
        let p_filename = p.file_name().unwrap().to_str().unwrap();
        let di = dist_infos.iter().find(|d| {
            d.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(p_filename)
        });
        // println!("p: {}", p_filename);
        // println!("di: {:?}", di);

        v.push((p.clone(), di.cloned()));
    }

    v
}
