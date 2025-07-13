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
) -> Vec<(PathBuf, Option<PathBuf>)> {
    let mut pairs = Vec::with_capacity(packages.len());

    for pkg in packages.into_iter() {
        let pkg_name = pkg.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let matched = dist_infos.iter().find_map(|di| {
            let di_fname = di.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Must end with ".dist-info"
            let without_suffix = di_fname.strip_suffix(".dist-info")?;

            // without_suffix is "{name}-{version}" – split at last hyphen
            let name_part = without_suffix
                .rfind('-')
                .map(|idx| &without_suffix[..idx])?;

            if name_part == pkg_name {
                Some(di.clone())
            } else {
                None
            }
        });

        pairs.push((pkg.clone(), matched));
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
