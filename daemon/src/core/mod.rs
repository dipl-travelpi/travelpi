//! Core of `travelpid`: API server on the Unix socket, event stream, simulation mode and
//! the pattern modules follow to report their status.

pub mod api;
pub mod reporter;
pub mod server;
pub mod simulation;
pub mod status;
