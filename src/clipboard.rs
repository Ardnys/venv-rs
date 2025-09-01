#[cfg(target_os = "linux")]
mod clipboard {
    use std::io::Write;
    use std::process::{Command, Stdio};

    pub fn copy_to_clipboard(cmd: &str) -> anyhow::Result<()> {
        let mut xclip = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = xclip.stdin.as_mut() {
            stdin.write_all(cmd.as_bytes())?;
        }
        xclip.wait()?;
        Ok(())
    }
}

#[cfg(windows)]
mod clipboard {
    use arboard::Clipboard;

    pub fn copy_to_clipboard(cmd: &str) -> color_eyre::Result<()> {
        Clipboard::new()?.set_text(cmd.to_string())?;
        Ok(())
    }
}

pub use clipboard::copy_to_clipboard;
