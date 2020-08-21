[![crates.io version](https://meritbadge.herokuapp.com/async-mpd)](https://crates.io/crates/async-mpd)
[![docs.rs](https://docs.rs/async-mpd/badge.svg)](https://docs.rs/async-mpd)

# async-mpd

 Runtime agnostic mpd client library

## Example:
```rust
// To use tokio you would do:
// use tokio as runtime;
use async_std as runtime;

#[runtime::main]
async fn main() -> Result<(), async_mpd::Error> {
    // Connect to server
    let mut mpd = async_mpd::MpdClient::new("localhost:6600").await?;

    // Get all tracks in the play queue
    let queue = mpd.queue().await?;

    // Print the queue
    for track in queue {
        println!(
            "{:3}: {} - {}",
            track.id.unwrap(),
            track.artist.unwrap_or_default(),
            track.title.unwrap_or_default(),
        );
    }

    // Play track nr 2 in the queue
    mpd.playid(2).await?;

    // Get and print the current server status
    println!("{:?}", mpd.status().await?);

    // Set the volume to 50%
    mpd.setvol(50).await?;

    // Stop playing
    mpd.stop().await?;

    Ok(())
}
```
