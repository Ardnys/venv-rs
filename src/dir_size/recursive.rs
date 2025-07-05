use super::chonk::Chonk;

use std::fs;
use std::path::Path;

struct RecursiveReader;

impl Chonk for RecursiveReader {
    fn get_dir_size(&self, dir: &Path) -> anyhow::Result<u64> {
        let mut size = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += self.get_dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
        Ok(size)
    }
}

// le tests
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::dir_size::chonk::Chonk;

    use super::RecursiveReader;

    #[test]
    fn test_recursive_dir_size() -> anyhow::Result<()> {
        let dir = Path::new("test_directories/basic");
        let method = RecursiveReader;
        let size = method.get_dir_size(dir)?;
        assert_eq!(size, 22);

        Ok(())
    }
}
