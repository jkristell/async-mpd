// To use tokio you would do:
// use tokio as runtime;
use async_std as runtime;

#[runtime::main]
async fn main() -> Result<(), async_mpd::Error> {
    femme::with_level(log::LevelFilter::Debug);

    // Connect to server
    let mut mpd = async_mpd::MpdClient::new();
    mpd.connect("localhost:6600").await?;

    // Get all tracks in the play queue
    let queue = mpd.queue().await?;

    // Print the queue
    for track in queue {
        println!(
            "{:3}: {} - {}",
            track.id.unwrap_or(0),
            track.artist.unwrap_or_else(|| "<NoArtist>".to_string()),
            track.title.unwrap_or_else(|| "<NoTitle>".to_string()),
        );
    }

    // Play track nr 2 in the queue
    mpd.playid(2).await?;

    // Get and print the current server status
    println!("{:?}", mpd.status().await?);

    println!("{:?}", mpd.stats().await?);

    // Set the volume to 50%
    mpd.setvol(50).await?;
    // Stop playing
    mpd.stop().await?;

    Ok(())
}
