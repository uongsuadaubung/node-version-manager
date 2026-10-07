use crate::anyhow;
use std::process::{Child, Command, Stdio};

const USER_AGENT: &str = "nvm-rust-gui";

pub fn create_curl() -> Command {
    let mut cmd = Command::new("curl");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut cmd = create_curl();
    let output = cmd.args(["-sL", "-A", USER_AGENT, url]).output()?;

    if !output.status.success() {
        anyhow::bail!("curl failed with status code: {:?}", output.status.code());
    }

    Ok(output.stdout)
}

pub fn get_content_length(url: &str) -> u64 {
    let mut cmd = create_curl();
    cmd.args(["-sIL", "-A", USER_AGENT, url])
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .filter(|l| l.to_ascii_lowercase().starts_with("content-length:"))
                .last()
                .and_then(|l| l.split_once(':'))
                .and_then(|(_, v)| v.trim().parse().ok())
        })
        .unwrap_or(0)
}

pub fn stream_download(url: &str) -> anyhow::Result<Child> {
    let mut cmd = create_curl();
    let child = cmd
        .args(["-sL", "-A", USER_AGENT, url])
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(child)
}

