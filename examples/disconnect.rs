use async_mpd::Error;
use std::time::Duration;

// To use tokio you would do:
// use tokio as runtime;
use async_std as runtime;

#[runtime::main]
async fn main() -> Result<(), Error> {
    // Connect to server
    let mut mpd = async_mpd::MpdClient::new("localhost:6600").await?;

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
        runtime::task::sleep(Duration::from_secs(120)).await;
    }

    Ok(())
}
