use std::path::Path;

pub trait Chonk {
    fn get_dir_size(&self, dir: &Path) -> color_eyre::Result<u64>;
    fn formatted_size(size: u64) -> String {
        let (num, suffix) = match size {
            0..1_000 => (size, "B"),
            1_000..1_000_000 => (size / 1_000, "KiB"),
            1_000_000..1_000_000_000 => (size / 1_000_000, "MiB"),
            1_000_000_000..1_000_000_000_000 => (size / 1_000_000_000, "GiB"),
            _ => (size, "spring rolls"),
        };
        format!("{} {}", num, suffix)
    }
}
