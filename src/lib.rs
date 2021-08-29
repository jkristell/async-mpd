//! # async-mpd
//!
//! Runtime agnostic Mpd client library for Rust
//!
//! ## Example:
//! ```
//! // Pick one
//! //use async_std as runtime;
//! use tokio as runtime;
//! use async_mpd::{MpdClient, cmd};
//!
//! #[runtime::main]
//! async fn main() -> Result<(), async_mpd::Error> {
//!     // Connect to server
//!     let mut mpd = MpdClient::new();
//!     mpd.connect("localhost:6600").await?;
//!
//!     // Get all tracks in the play queue and display them
//!     let queue = mpd.queue().await?;
//!     for track in queue {
//!         println!("{:?} - {:?}", track.artist, track.title);
//!     }
//!
//!     // Play track nr 2 in the queue
//!     mpd.playid(2).await?;
//!
//!     // Get and print the current server status using the command api
//!     let status = mpd.exec(cmd::Status).await?;
//!     println!("{:?}", status);
//!
//!     // Set the volume to 50%
//!     mpd.setvol(50).await?;
//!
//!     Ok(())
//! }
//! ```
//!

#[cfg(feature = "client")]
mod client;
mod protocol;

#[cfg(feature = "client")]
pub use client::*;
pub use protocol::*;
