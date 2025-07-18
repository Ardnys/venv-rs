use super::chonk::Chonk;

use std::fs;
use std::path::{Path, PathBuf};

pub struct IterativeReader;

impl Chonk for IterativeReader {
    fn get_dir_size(&self, dir: &Path) -> color_eyre::Result<u64> {
        // very iterative lol
        if !dir.is_dir() {
            return Ok(dir.metadata().map(|m| m.len()).unwrap_or(0));
        }

        let size = fs::read_dir(dir)?
            .filter_map(|item| item.map(|e| e.path()).ok())
            .fold(0, |acc, path: PathBuf| {
                if path.is_dir() && !path.is_symlink() {
                    acc + self.get_dir_size(&path).unwrap()
                } else {
                    acc + path.metadata().unwrap().len()
                }
            });
        Ok(size)
    }
}

// le tests
// #[cfg(test)]
// mod tests {
//     use std::path::Path;
//
//     use crate::dir_size::chonk::Chonk;
//
//     use super::IterativeReader;
//
//     #[test]
//     fn test_iterative_dir_size() -> color_eyre::Result<()> {
//         let dir = Path::new("test_directories/basic");
//         let method = IterativeReader;
//         let size = method.get_dir_size(dir)?;
//         assert_eq!(size, 22);
//
//         Ok(())
//     }
// }
