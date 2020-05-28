[![crates.io version](https://meritbadge.herokuapp.com/async-mpd)](https://crates.io/crates/async-mpd)
[![docs.rs](https://docs.rs/async-mpd/badge.svg)](https://docs.rs/async-mpd)

# Async-mpd

Async-std based Mpd client library

## Example:
```rust
use async_mpd::MpdClient;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    // Connect to server
    let mut mpd = MpdClient::new("localhost:6600").await?;

    // Get all tracks in the play queue and display them
    let queue = mpd.queue().await?;
    for track in queue {
        println!("{} - {}", track.artist, track.title);
    }

    // Play track nr 2 in the queue
    mpd.playid(2).await?;

    // Get and print the current server status
    println!("{:?}", mpd.status().await?);

    // Set the volume to 50%
    mpd.setvol(50).await?;

    Ok(())
}
```
