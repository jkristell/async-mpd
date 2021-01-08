use async_mpd::{Command, CommandResponse, Error, MpdClient};
use structopt::StructOpt;

// To use tokio you would do:
// use tokio as runtime;
use async_std as runtime;
use std::time::Duration;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(name = "host", default_value = "localhost", long)]
    /// Hostname
    host: String,
    #[structopt(name = "port", default_value = "6600", long)]
    /// Port
    port: String,
}

#[runtime::main]
async fn main() -> Result<(), Error> {
    femme::with_level(log::LevelFilter::Debug);

    // Example that plays 10s of every song in the playlist using the command api

    let opt = Opt::from_args();

    let addr = format!("{}:{}", opt.host, opt.port);
    let mut mpd = MpdClient::new(&addr).await?;

    let mut even = true;

    execute(&mut mpd, &Command::PlayId(1)).await?;

    loop {
        let cmd = if even { Command::Status } else { Command::Next };

        even = !even;

        let res = execute(&mut mpd, &cmd).await?;

        match res {
            CommandResponse::Status(s) => {
                println!("Play state: {:?}", s.state)
            }
            CommandResponse::Ok => {
                println!("Next")
            }
            _ => unreachable!(),
        }

        runtime::task::sleep(Duration::from_secs(5)).await;
    }
}

async fn execute(
    mpd: &mut MpdClient,
    cmd: &Command<'_>,
) -> Result<CommandResponse, async_mpd::Error> {
    let mut tries = 0;

    let ret = loop {
        match mpd.cmd(cmd).await {
            Ok(resp) => break Ok(resp),
            Err(Error::Disconnected) => {
                println!("Server disconnected. Trying to reconnect");
                mpd.reconnect().await?;
            }
            Err(other) => {
                println!("Error: {:?}", other);
                break Err(async_mpd::Error::Disconnected);
            }
        }

        tries += 1;

        if tries > 3 {
            break Err(Error::Disconnected);
        }
    };

    ret
}
