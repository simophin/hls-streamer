use std::ffi::OsStr;
use std::io;
use std::time::Duration;

use async_std::path::{Path, PathBuf};
use async_std::process::Command;
use async_std::task::{sleep, spawn};
use futures::{
    channel::mpsc::channel as new_channel, channel::mpsc::Sender, select, FutureExt, SinkExt,
    StreamExt,
};
use tide::{log, Body, Request, Response, StatusCode};

type KeepAlive = ();

const PLAYLIST_WAIT_INTERVAL: Duration = Duration::from_secs(2);
const PLAYLIST_WAIT_MAX_RETRIES: usize = 20;

fn wait_and_serve_ffmpeg(
    output_dir: &Path,
    timeout: Duration,
) -> anyhow::Result<Sender<KeepAlive>> {
    let output_file = output_dir.join("master.m3u8");

    let (keepalive_tx, mut keepalive_rx) = new_channel::<KeepAlive>(2);
    spawn(async move {
        while let Some(_) = keepalive_rx.next().await {
            let mut cmd = Command::new("ffmpeg");

            for opt in std::env::var("FFMPEG_INPUT")
                .unwrap_or(String::new())
                .split(" ")
            {
                cmd.arg(opt);
            }

            cmd.args(&[
                "-f",
                "hls",
                "-hls_time",
                "2",
                "-hls_list_size",
                "5",
                "-hls_flags",
                "delete_segments+append_list",
            ]);

            cmd.arg(&output_file);
            cmd.kill_on_drop(true);

            let _ = std::fs::remove_file(&output_file);

            let mut child = match cmd.spawn() {
                Ok(c) => {
                    log::info!("Child process started: {:?}", c);
                    c
                }
                Err(e) => {
                    log::error!("Error spawning ffmpeg: {:?}", e);
                    continue;
                }
            };

            loop {
                select! {
                    status = child.status().fuse() => {
                        log::info!("FFMPEG exited with status: {:?}", status);
                        break;
                    }

                    _ = keepalive_rx.next() => {
                        log::info!("Keeping alive");
                    }

                     _ = sleep(timeout).fuse() => {
                        log::info!("Timeout!");
                        break;
                    }
                }
            }

            drop(child);
            log::info!("FFMPEG terminated");
        }
    });
    Ok(keepalive_tx)
}

#[derive(Clone)]
struct AppState {
    cmd_tx: Sender<KeepAlive>,
    output_dir: PathBuf,
}

async fn serve_http(req: Request<AppState>) -> tide::Result {
    let _ = req.state().cmd_tx.clone().send(()).await;

    let rel_path = match req.url().path() {
        s if s == "/" => "index.html",
        s if s.starts_with("/") => &s[1..],
        s => s,
    };

    if rel_path == "index.html" {
        return Ok(Response::builder(StatusCode::Ok)
            .content_type("text/html")
            .body(Body::from(&include_bytes!("index.html")[0..]))
            .build());
    }

    let file_path = req.state().output_dir.join(rel_path);

    if file_path.file_name().and_then(OsStr::to_str) == Some("master.m3u8") {
        // Wait until we have this file
        let mut i = 0;
        while !file_path.is_file().await && i < PLAYLIST_WAIT_MAX_RETRIES {
            log::info!("Requesting playlist file but no list is generated. Waiting...");
            sleep(PLAYLIST_WAIT_INTERVAL).await;
            i = i + 1;
        }
    }

    match Body::from_file(&file_path).await {
        Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            log::warn!("File not found: {:?}", file_path);
            Ok(Response::new(StatusCode::NotFound))
        }
        Err(e) => Err(e.into()),
    }
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    log::start();
    let output_dir = PathBuf::from(std::env::var("HLS_DIR").expect("HLS_DIR to be present"));

    let mut app = tide::with_state(AppState {
        cmd_tx: wait_and_serve_ffmpeg(
            &output_dir,
            Duration::from_secs(
                std::env::var("TIMEOUT_SECONDS")
                    .unwrap_or("120".to_string())
                    .parse()
                    .expect("TIMEOUT_SECONDS to be a number"),
            ),
        )?,
        output_dir,
    });

    app.at("/").get(serve_http);
    app.at("/*").get(serve_http);

    app.listen(format!(
        "{}:{}",
        std::env::var("LISTEN_ADDRESS").unwrap_or("127.0.0.1".to_string()),
        std::env::var("LISTEN_PORT").unwrap_or("8989".to_string()),
    ))
    .await?;
    Ok(())
}
