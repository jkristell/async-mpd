[![crates.io version](https://img.shields.io/crates/v/async-mpd)](https://crates.io/crates/async-mpd)
[![docs.rs](https://docs.rs/async-mpd/badge.svg)](https://docs.rs/async-mpd)

# async-mpd

Runtime agnostic Mpd client library for Rust

## Example:
```rust
use tokio as runtime;
// For async-std instead
//use async_std as runtime;
use async_mpd::{MpdClient, cmd};

#[runtime::main]
async fn main() -> Result<(), async_mpd::Error> {
    // Connect to server
    let mut mpd = MpdClient::new();
    mpd.connect("localhost:6600").await?;

    // Get all tracks in the play queue and display them
    let queue = mpd.queue().await?;
    for track in queue {
        println!("{:?} - {:?}", track.artist, track.title);
    }

    // Play track nr 2 in the queue
    mpd.playid(2).await?;

    // Get and print the current server status using the command api
    let status = mpd.exec(cmd::Status).await?;
    println!("{:?}", status);

    // Set the volume to 50%
    mpd.setvol(50).await?;

    Ok(())
}
```
