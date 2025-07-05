use anyhow::Result;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::chonk::Chonk;

pub struct ParallelReader;

impl Chonk for ParallelReader {
    fn get_dir_size(&self, dir: &Path) -> Result<u64> {
        let entries: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        let size: u64 = entries
            .par_iter()
            .map(|path| {
                if path.is_dir() {
                    self.get_dir_size(path).unwrap_or(0)
                } else {
                    path.metadata().map(|m| m.len()).unwrap_or(0)
                }
            })
            .sum();

        Ok(size)
    }
}

// le tests
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::dir_size::chonk::Chonk;

    use super::ParallelReader;

    #[test]
    fn test_parallel_dir_size() -> anyhow::Result<()> {
        let dir = Path::new("test_directories/basic");
        let method = ParallelReader;
        let size = method.get_dir_size(dir)?;
        assert_eq!(size, 22);

        Ok(())
    }
}
