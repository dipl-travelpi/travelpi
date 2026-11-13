//! `travelpid`, the system daemon of TravelPi.

mod core;

use std::path::PathBuf;
use std::process::ExitCode;

use crate::core::api::{Sections, router};
use crate::core::reporter::{channel, spawn_module};
use crate::core::status::{ApStatus, UplinkStatus};
use crate::core::{server, simulation};

/// Socket path on the device. The folder is created by systemd.
const DEFAULT_SOCKET: &str = "/run/travelpid/api.sock";
/// Socket path in simulation mode, in the current folder.
const SIMULATION_SOCKET: &str = "travelpid.sock";

const USAGE: &str = "\
Usage: travelpid [--simulate] [--socket <path>]

  --simulate       report made-up values instead of using the hardware
  --socket <path>  Unix socket to listen on
                   (default: /run/travelpid/api.sock, with --simulate: ./travelpid.sock)
  --help           show this text";

struct Options {
    simulate: bool,
    socket: PathBuf,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut simulate = false;
    let mut socket = None;
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--simulate" => simulate = true,
            "--socket" => {
                let path = args.next().ok_or("--socket needs a path")?;
                socket = Some(PathBuf::from(path));
            }
            "--help" | "-h" => return Err(USAGE.to_string()),
            other => return Err(format!("unknown argument: {other}\n\n{USAGE}")),
        }
    }
    let default = if simulate {
        SIMULATION_SOCKET
    } else {
        DEFAULT_SOCKET
    };
    Ok(Options {
        simulate,
        socket: socket.unwrap_or_else(|| PathBuf::from(default)),
    })
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();

    let options = match parse_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let (ap_reporter, ap) = channel::<ApStatus>();
    let (uplink_reporter, uplink) = channel::<UplinkStatus>();

    // A section whose reporter is dropped would end the event stream, so the reporters
    // without a module are kept here until the daemon stops.
    let mut unused_reporters = None;
    if options.simulate {
        tracing::info!("simulation mode");
        spawn_module(ap_reporter, simulation::ap);
        spawn_module(uplink_reporter, simulation::uplink);
    } else {
        // The real modules (ap, uplink) are started here once they exist.
        tracing::warn!("no modules yet; every section keeps its initial state");
        unused_reporters = Some((ap_reporter, uplink_reporter));
    }

    let result = server::serve(router(Sections { ap, uplink }), &options.socket).await;
    drop(unused_reporters);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "could not serve the API");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        parse_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn default_socket_depends_on_mode() {
        assert_eq!(parse(&[]).unwrap().socket, PathBuf::from(DEFAULT_SOCKET));
        assert_eq!(
            parse(&["--simulate"]).unwrap().socket,
            PathBuf::from(SIMULATION_SOCKET)
        );
    }

    #[test]
    fn socket_can_be_set() {
        let options = parse(&["--socket", "/tmp/x.sock", "--simulate"]).unwrap();
        assert!(options.simulate);
        assert_eq!(options.socket, PathBuf::from("/tmp/x.sock"));
    }

    #[test]
    fn bad_arguments_are_rejected() {
        assert!(parse(&["--socket"]).is_err());
        assert!(parse(&["--fast"]).is_err());
    }
}
