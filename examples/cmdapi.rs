use async_mpd::{cmd, Error, MpdClient, ResponseHandler, WrappedResponse};
use structopt::StructOpt;

// To use tokio you would do:
// use tokio as runtime;
use async_mpd::cmd::MpdCmd;
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
    femme::with_level(log::LevelFilter::Warn);

    let opt = Opt::from_args();

    let addr = format!("{}:{}", opt.host, opt.port);
    let mut mpd = MpdClient::new();

    mpd.connect(&addr).await?;

    // Response with known type
    let _status = dispatcher(&mut mpd, cmd::Status).await?;

    let mut flip = true;
    loop {
        let resp = if flip {
            enum_dispatcher(&mut mpd, cmd::Status).await?
        } else {
            enum_dispatcher(&mut mpd, cmd::Stats).await?
        };

        match resp {
            WrappedResponse::Status(s) => println!("{:?}", s),
            WrappedResponse::Stats(s) => println!("{:?}", s),
            _ => (),
        };

        flip = !flip;
        runtime::task::sleep(Duration::from_secs(5)).await;
    }
}

async fn enum_dispatcher<C: MpdCmd + Copy>(
    mpd: &mut MpdClient,
    cmd: C,
) -> Result<WrappedResponse, async_mpd::Error> {
    let mut tries = 0;

    let ret = loop {
        match mpd.exec_wrapped(cmd).await {
            Ok(resp) => break Ok(resp),
            Err(Error::Disconnected) => {
                println!("Server disconnected. Trying to reconnect");
                mpd.reconnect().await?;
            }
            Err(other) => {
                println!("Error: {:?}", other);
                break Err(other);
            }
        }

        tries += 1;

        if tries == 3 {
            break Err(Error::Disconnected);
        }
    };

    ret
}

async fn dispatcher<C: MpdCmd + Copy>(
    mpd: &mut MpdClient,
    cmd: C,
) -> Result<<C::Handler as ResponseHandler>::Response, async_mpd::Error> {
    let mut tries = 0;

    let ret = loop {
        match mpd.exec(cmd).await {
            Ok(resp) => break Ok(resp),
            Err(Error::Disconnected) => {
                println!("Server disconnected. Trying to reconnect");
                mpd.reconnect().await?;
            }
            Err(other) => {
                println!("Error: {:?}", other);
                break Err(other);
            }
        }

        tries += 1;

        if tries == 3 {
            break Err(Error::Disconnected);
        }
    };

    ret
}
