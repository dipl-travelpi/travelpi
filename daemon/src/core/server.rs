//! Listening on the Unix socket.

use std::io;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Path;

use axum::Router;
use tokio::net::{UnixListener, UnixStream};

/// Permissions of the socket: owner and group may connect, nobody else.
const SOCKET_MODE: u32 = 0o660;

/// Serves `app` on the socket at `path` until a stop signal arrives, then removes the
/// socket file.
pub async fn serve(app: Router, path: &Path) -> io::Result<()> {
    remove_stale_socket(path).await?;
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(SOCKET_MODE))?;
    tracing::info!(socket = %path.display(), "listening");

    // No graceful shutdown: open event streams never end by themselves.
    let result = tokio::select! {
        result = axum::serve(listener, app) => result,
        () = stop_signal() => {
            tracing::info!("stopping");
            Ok(())
        }
    };
    let _ = std::fs::remove_file(path);
    result
}

/// A socket file left over from a crashed run would make `bind` fail. Remove it, but
/// refuse to start if another `travelpid` is still listening on it.
async fn remove_stale_socket(path: &Path) -> io::Result<()> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.file_type().is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} exists and is not a socket", path.display()),
        ));
    }
    if UnixStream::connect(path).await.is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            format!("another travelpid is listening on {}", path.display()),
        ));
    }
    std::fs::remove_file(path)
}

/// Waits for Ctrl+C or SIGTERM (sent by systemd on stop).
async fn stop_signal() {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("can listen for SIGTERM");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
}
