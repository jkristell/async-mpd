use async_mpd::{Error, MpdClient};
use std::time::Duration;

// To use tokio you would do:
// use tokio as runtime;
use async_std as runtime;

#[runtime::main]
async fn main() -> Result<(), Error> {
    femme::with_level(log::LevelFilter::Debug);
    // Connect to server
    let mut mpd = MpdClient::new();

    let addr = "localhost:6600";

    mpd.connect(addr).await?;

    loop {
        let status = match mpd.status().await {
            Ok(status) => status,
            Err(Error::Disconnected) => {
                println!("Server disconnected. Reconnecting");
                mpd.reconnect().await?;
                continue;
            }
            Err(other) => {
                println!("Error: {:?}", other);
                break;
            }
        };

        println!("Status: {:?}", status);

        // The mpd server closes connection without activity after a configurable amount of time
        runtime::task::sleep(Duration::from_secs(12)).await;
    }

    Ok(())
}
