use std::path::Path;

pub trait Chonk {
    fn get_dir_size(&self, dir: &Path) -> anyhow::Result<u64>;
}
