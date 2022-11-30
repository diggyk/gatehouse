use async_recursion::async_recursion;
use etcd_client::{Client, DeleteOptions};
use std::convert::From;
use std::process::{exit, Stdio};
use tokio::fs::read_dir;
use tokio::process::Command;

use gatehouse::StorageType;

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

async fn clear_etcd(url: &str) {
    let client = match Client::connect([url], None).await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Could not connect to Etcd storage: {err}");
            exit(1);
        }
    };

    let basepath = String::from("/gatehouse");

    client
        .kv_client()
        .delete(basepath, Some(DeleteOptions::new().with_prefix()))
        .await
        .unwrap();
}

/// Runs the server for us to test against. We should make sure this runs with a file backend
/// and that the local file storage is cleaned so it starts empty
pub async fn run_server() {
    let storage: StorageType = std::env::var("GATESTORAGE")
        .unwrap_or_else(|_| String::from("file:/tmp/gatehouse"))
        .into();

    match storage {
        StorageType::FileSystem(path) => clear_dir(&path).await,
        StorageType::Etcd(url) => clear_etcd(&url).await,
        _ => (),
    }

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
