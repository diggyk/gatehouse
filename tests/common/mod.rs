use async_recursion::async_recursion;
use std::process::Stdio;
use tokio::fs::read_dir;
use tokio::process::Command;

#[async_recursion]
async fn clear_dir(path: &str) {
    let dir = read_dir(path).await;
    if dir.is_err() {
        // doesn't exist maybe so let's move on
        return;
    }
    let mut dir = dir.unwrap();

    while let Some(entry) = dir
        .next_entry()
        .await
        .expect("Could not read entry in tmp dir")
    {
        let metadata = entry
            .metadata()
            .await
            .expect("Could not read metadata for an entry in tmp dir");

        if metadata.is_file() {
            tokio::fs::remove_file(entry.path())
                .await
                .expect("Could not delete file in tmp dir");
        } else if metadata.is_dir() {
            if let Some(subpath) = entry.path().to_str() {
                clear_dir(subpath).await;
            }
        }
    }
}

/// Runs the server for us to test against. We should make sure this runs with a file backend
/// and that the local file storage is cleaned so it starts empty
pub async fn run_server() {
    clear_dir("/tmp/gatehouse").await;

    Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("gatehouse-server")
        .kill_on_drop(true)
        .stdout(Stdio::null())
        .spawn()
        .expect("Could not start server")
        .wait()
        .await
        .expect("Server failed");
}
