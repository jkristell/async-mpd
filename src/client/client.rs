use async_net::{AsyncToSocketAddrs, TcpStream};
use futures_lite::io::BufReader;

use std::{io, net::SocketAddr};

use crate::client::{Command, CommandResponse};
use crate::{client::resp::MixedResponse, client::Filter, Stats, Status, Subsystem, Track};

/// Error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Server failed to parse the command
    #[error("Invalid command or arguments")]
    CommandError { msg: String },

    /// The server closed the connection
    #[error("The server closed the connection")]
    Disconnected,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] io::Error),

    /// Failed to parse the reply the server sent
    #[error("Invalid reply to command")]
    ResponseError { reply: String, errmsg: String },

    /// Generic unexpected response error
    #[error("invalid value error")]
    ValueError { msg: String },
}

/// Mpd Client
pub struct MpdClient {
    bufreader: BufReader<TcpStream>,
    version: String,
    server_address: SocketAddr,
}

impl MpdClient {
    /// Create a new MpdClient and connect to `addr`
    pub async fn new<A: AsyncToSocketAddrs>(addr: A) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr).await?;
        let server_address = stream.peer_addr()?;
        let bufreader = BufReader::new(stream);

        let mut s = Self {
            bufreader,
            version: String::new(),
            server_address,
        };

        s.read_version().await?;

        Ok(s)
    }

    /// Reconnect to server
    pub async fn reconnect(&mut self) -> Result<(), Error> {
        let stream = TcpStream::connect(self.server_address).await?;
        let bufreader = BufReader::new(stream);
        self.bufreader = bufreader;
        self.read_version().await?;

        Ok(())
    }

    async fn read_version(&mut self) -> Result<(), Error> {
        self.version = crate::io::read_resp_line(&mut self.bufreader).await?;
        log::debug!("Connected: {}", self.version);
        Ok(())
    }

    /// Get stats on the music database
    pub async fn stats(&mut self) -> Result<Stats, Error> {
        self.cmd_into(&Command::Stats).await
    }

    pub async fn status(&mut self) -> Result<Status, Error> {
        self.cmd_into(&Command::Status).await
    }

    pub async fn update(&mut self, path: Option<&str>) -> Result<i32, Error> {
        self.cmd_into(&Command::Update(path)).await
    }

    pub async fn rescan(&mut self, path: Option<&str>) -> Result<i32, Error> {
        self.cmd_into(&Command::Rescan(path)).await
    }

    pub async fn idle(&mut self) -> Result<Subsystem, Error> {
        self.cmd_into(&Command::Idle).await
    }

    pub async fn noidle(&mut self) -> Result<(), Error> {
        self.cmd_into(&Command::NoIdle).await
    }

    pub async fn setvol(&mut self, volume: u32) -> Result<(), Error> {
        self.cmd_into(&Command::SetVol(volume)).await
    }

    pub async fn repeat(&mut self, repeat: bool) -> Result<(), Error> {
        self.cmd_into(&Command::Repeat(repeat)).await
    }

    pub async fn random(&mut self, random: bool) -> Result<(), Error> {
        self.cmd_into(&Command::Random(random)).await
    }

    pub async fn consume(&mut self, consume: bool) -> Result<(), Error> {
        self.cmd_into(&Command::Consume(consume)).await
    }

    // Playback controls

    pub async fn play(&mut self) -> Result<(), Error> {
        self.play_pause(true).await
    }

    pub async fn playid(&mut self, id: u32) -> Result<(), Error> {
        self.cmd_into(&Command::PlayId(id)).await
    }

    pub async fn pause(&mut self) -> Result<(), Error> {
        self.play_pause(false).await
    }

    pub async fn play_pause(&mut self, play: bool) -> Result<(), Error> {
        self.cmd_into(&Command::PlayPaus(!play)).await
    }

    pub async fn next(&mut self) -> Result<(), Error> {
        self.cmd_into(&Command::Next).await
    }

    pub async fn prev(&mut self) -> Result<(), Error> {
        self.cmd_into(&Command::Prev).await
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.cmd_into(&Command::Stop).await
    }

    //
    // Music database commands
    //

    pub async fn listall(&mut self, path: Option<String>) -> Result<Vec<String>, Error> {
        self.cmd_into(&Command::Listall(path)).await
    }

    pub async fn listallinfo(&mut self, path: Option<&str>) -> Result<Vec<MixedResponse>, Error> {
        self.cmd_into(&Command::ListallInfo(path)).await
    }

    // Queue handling commands

    pub async fn queue_add(&mut self, path: &str) -> Result<(), Error> {
        self.cmd_into(&Command::QueueAdd(path)).await
    }

    pub async fn queue_clear(&mut self) -> Result<(), Error> {
        self.cmd_into(&Command::QueueClear).await
    }

    pub async fn queue(&mut self) -> Result<Vec<Track>, Error> {
        self.cmd_into(&Command::PlaylistInfo).await
    }

    /// # Example
    /// ```
    /// use async_mpd::{MpdClient, Error, Tag, Filter, ToFilterExpr};
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<(), Error> {
    ///     // Connect to server
    ///     let mut mpd = MpdClient::new("localhost:6600").await?;
    ///
    ///     let mut filter = Filter::new()
    ///         .and(Tag::Artist.equals("The Beatles"))
    ///         .and(Tag::Album.contains("White"));
    ///
    ///     let res = mpd.search(&filter).await?;
    ///     println!("{:?}", res);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn search(&mut self, filter: &Filter) -> Result<Vec<Track>, Error> {
        self.cmd_into(&Command::Search(filter.to_query())).await
    }

    pub async fn cmd(&mut self, cmd: &Command<'_>) -> Result<CommandResponse, Error> {
        let cmdline = cmd.line();
        log::debug!("Sending cmdline: {}", cmdline);

        crate::io::send_command(&cmdline, &mut self.bufreader).await?;

        crate::io::handle_resp(cmd, &mut self.bufreader).await
    }

    pub async fn cmd_into<T: From<CommandResponse>>(
        &mut self,
        cmd: &Command<'_>,
    ) -> Result<T, Error> {
        self.cmd(cmd).await.map(Into::into)
    }
}
