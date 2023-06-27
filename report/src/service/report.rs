use std::process::Stdio;

use tokio::{io::AsyncWriteExt, process::Command};

pub async fn create_pandoc_report(mut md: String) -> anyhow::Result<String> {
    let mut child = Command::new("pandoc")
        .arg("-f")
        .arg("markdown")
        .arg("-t")
        .arg("latex")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(md.as_bytes()).await?;
    drop(stdin);

    let report = String::from_utf8(child.wait_with_output().await?.stdout)?;
    Ok(report)
}
