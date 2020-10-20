//! # async-mpd
//!
//! Runtime agnostic mpd client library for Rust
//!
//! ## Example:
//! ```
//! use async_std as runtime;
//! use async_mpd::MpdClient;
//!
//! #[runtime::main]
//! async fn main() -> Result<(), async_mpd::Error> {
//!     // Connect to server
//!     let mut mpd = MpdClient::new("localhost:6600").await?;
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
//!     // Get and print the current server status
//!     println!("{:?}", mpd.status().await?);
//!
//!     // Set the volume to 50%
//!     mpd.setvol(50).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Using tokio:
//! ```
//! use tokio as runtime;
//! use async_mpd::MpdClient;
//!
//! #[runtime::main]
//! async fn main() -> Result<(), async_mpd::Error> {
//!     // Connect to server
//!     let mut mpd = MpdClient::new("localhost:6600").await?;
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
//!     // Get and print the current server status
//!     println!("{:?}", mpd.status().await?);
//!
//!     // Set the volume to 50%
//!     mpd.setvol(50).await?;
//!
//!     // Pause
//!     mpd.pause().await?;
//!
//!     Ok(())
//! }
//! ```

mod protocol;
pub use protocol::*;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;
