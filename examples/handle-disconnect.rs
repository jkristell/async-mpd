use async_mpd::Error;
use std::time::Duration;

#[async_std::main]
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
        async_std::task::sleep(Duration::from_secs(120)).await;
    }

    Ok(())
}
