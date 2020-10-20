use async_mpd::{Error, Filter, MixedResponse, MpdClient, Tag, ToFilterExpr};
use structopt::StructOpt;

// To use tokio you would do:
// use tokio as runtime;
use async_std as runtime;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(name = "host", default_value = "localhost", long)]
    /// Hostname
    host: String,
    #[structopt(name = "port", default_value = "6600", long)]
    /// Port
    port: String,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Status
    Status,
    /// Add files to queue
    Add {
        path: String,
    },
    Play,
    Playid {
        id: u32,
    },
    Pause,
    Stop,
    /// Next
    Next,
    /// Clear the queue
    Clear,
    Setvol {
        vol: u32,
    },
    Listall {
        path: Option<String>,
    },
    Queue,
    Idle,
    Stats,
    Update,
    Rescan,
    Search {
        artist: Option<String>,
        album: Option<String>,
    },
    Lsinfo {
        path: Option<String>,
    },
}

#[runtime::main]
async fn main() -> Result<(), Error> {
    femme::with_level(log::LevelFilter::Debug);

    let opt = Opt::from_args();

    let addr = format!("{}:{}", opt.host, opt.port);
    let mut client = MpdClient::new(&addr).await?;

    match opt.cmd {
        Command::Status => {
            let s = client.status().await?;
            println!("{:?}", s);
        }
        Command::Stats => {
            let s = client.stats().await?;
            println!("{:?}", s);
        }
        Command::Add { path } => {
            client.queue_add(&path).await?;
        }
        Command::Next => {
            client.next().await?;
        }
        Command::Play => {
            client.play().await?;
        }
        Command::Playid { id } => {
            client.playid(id).await?;
        }
        Command::Pause => {
            client.pause().await?;
        }
        Command::Stop => {
            client.stop().await?;
        }
        Command::Clear => {
            client.queue_clear().await?;
        }
        Command::Setvol { vol } => {
            client.setvol(vol).await?;
        }
        Command::Listall { path } => {
            let files = client.listall(path).await?;
            for f in files {
                println!("{}", f);
            }
        }
        Command::Queue => {
            let queue = client.queue().await?;
            for song in queue {
                println!("{:?}:\t{:?} - {:?}", song.pos, song.artist, song.title);
            }
        }
        Command::Idle => loop {
            let r = client.idle().await?;
            println!("{:?}", r);
        },
        Command::Update => {
            let dbv = client.update(None).await?;
            println!("Update id: {}", dbv);
        }
        Command::Rescan => {
            let dbv = client.rescan(None).await?;
            println!("Rescan id: {}", dbv);
        }
        Command::Search { artist, album } => {
            use Tag::*;

            let mut filter = Filter::new();

            if let Some(artist) = artist {
                filter = filter.and(Artist.contains(&artist));
            }

            if let Some(album) = album {
                filter = filter.and(Album.contains(&album));
            }

            let res = client.search(&filter).await?;

            println!("{:?}", res);
        }
        Command::Lsinfo { path } => {
            let res = client.listallinfo(path.as_deref()).await?;

            for t in res {
                match t {
                    MixedResponse::File(t) => println!("File: {}", t.file),
                    MixedResponse::Directory(d) => println!("directory: {}", d.path),
                    MixedResponse::Playlist(pl) => println!("playlist: {}", pl.path),
                }
            }
        }
    }

    Ok(())
}
