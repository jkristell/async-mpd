use async_net::{AsyncToSocketAddrs, TcpStream};
use futures_lite::{io::BufReader, AsyncWriteExt};

use crate::resp::EnumResponse;
use crate::{
    client::resp::{
        handlers::ResponseHandler,
        read_resp_line,
        respmap_handlers::{ListallResponse, ListallinfoResponse},
    },
    cmd::{self, MpdCmd},
    DatabaseVersion, Error, Filter, Stats, Status, Subsystem, Track,
};
use std::net::SocketAddr;

/// Mpd Client
pub struct MpdClient {
    /// Buffered Stream
    stream: Option<BufReader<TcpStream>>,
    // Addr
    addr: Option<SocketAddr>,
}

impl MpdClient {
    /// Create a new MpdClient
    pub fn new() -> Self {
        Self {
            stream: None,
            addr: None,
        }
    }

    pub async fn connect<A: AsyncToSocketAddrs>(&mut self, addr: A) -> Result<String, Error> {
        let stream = TcpStream::connect(addr).await?;
        // Save the resolved adress for reconnect
        let sock_addr = stream.peer_addr()?;

        let reader = BufReader::new(stream);

        log::debug!("server: {:?}", sock_addr);

        self.stream = Some(reader);
        self.addr = Some(sock_addr);

        // After connect, the server replies with a a version reply
        Ok(self.read_version().await?)
    }

    pub async fn reconnect(&mut self) -> Result<(), Error> {
        if let Some(addr) = self.addr {
            log::debug!("Reconnection to: {:?}", addr);
            self.connect(addr).await.map(|_| ())
        } else {
            log::warn!("Reconnect without previous connection");
            Err(Error::Disconnected)
        }
    }

    async fn read_version(&mut self) -> Result<String, Error> {
        let br = self.stream.as_mut().ok_or(Error::Disconnected)?;

        let version = read_resp_line(br).await?;
        log::debug!("Connected: {}", version);
        Ok(version)
    }

    /// Get stats on the music database
    pub async fn stats(&mut self) -> Result<Stats, Error> {
        self.exec(cmd::Stats).await
    }

    pub async fn status(&mut self) -> Result<Status, Error> {
        let status = self.exec(cmd::Status).await?;
        Ok(status)
    }

    pub async fn update(&mut self, path: Option<&str>) -> Result<DatabaseVersion, Error> {
        self.exec(cmd::Update(path)).await
    }

    pub async fn rescan(&mut self, path: Option<&str>) -> Result<DatabaseVersion, Error> {
        self.exec(cmd::Rescan(path)).await
    }

    pub async fn idle(&mut self) -> Result<Subsystem, Error> {
        self.exec(cmd::Idle).await
    }

    pub async fn noidle(&mut self) -> Result<(), Error> {
        self.exec(cmd::NoIdle).await
    }

    pub async fn setvol(&mut self, volume: u32) -> Result<(), Error> {
        self.exec(cmd::Setvol(volume)).await
    }

    pub async fn repeat(&mut self, repeat: bool) -> Result<(), Error> {
        self.exec(cmd::Repeat(repeat)).await
    }

    pub async fn random(&mut self, random: bool) -> Result<(), Error> {
        self.exec(cmd::Random(random)).await
    }

    pub async fn consume(&mut self, consume: bool) -> Result<(), Error> {
        self.exec(cmd::Consume(consume)).await
    }

    // Playback controls

    pub async fn play(&mut self) -> Result<(), Error> {
        self.play_pause(true).await
    }

    pub async fn playid(&mut self, id: u32) -> Result<(), Error> {
        self.exec(cmd::PlayId(id)).await
    }

    pub async fn pause(&mut self) -> Result<(), Error> {
        self.play_pause(false).await
    }

    pub async fn play_pause(&mut self, play: bool) -> Result<(), Error> {
        self.exec(cmd::PlayPause(!play)).await
    }

    pub async fn next(&mut self) -> Result<(), Error> {
        self.exec(cmd::Next).await
    }

    pub async fn prev(&mut self) -> Result<(), Error> {
        self.exec(cmd::Prev).await
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.exec(cmd::Stop).await
    }

    //
    // Music database commands
    //

    pub async fn listall(&mut self, path: Option<&str>) -> Result<ListallResponse, Error> {
        self.exec(cmd::Listall(path)).await
    }

    pub async fn listallinfo(&mut self, path: Option<&str>) -> Result<ListallinfoResponse, Error> {
        self.exec(cmd::ListallInfo(path)).await
    }

    // Queue handling commands

    pub async fn queue_add(&mut self, path: &str) -> Result<(), Error> {
        self.exec(cmd::QueueAdd(path)).await
    }

    pub async fn queue_clear(&mut self) -> Result<(), Error> {
        self.exec(cmd::QueueClear).await
    }

    pub async fn queue(&mut self) -> Result<Vec<Track>, Error> {
        self.exec(cmd::PlaylistInfo).await
    }

    /// # Example
    /// ```
    /// use async_mpd::{MpdClient, Error, Tag, Filter, ToFilterExpr};
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<(), Error> {
    ///     // Connect to server
    ///     let mut mpd = MpdClient::new();
    ///     mpd.connect("localhost:6600").await?;
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
        self.exec(cmd::Search(filter.to_query().as_deref())).await
    }

    /*
    pub async fn exec<C>(
        &mut self,
        cmd: C,
    ) -> Result<<C::Handler as ResponseHandler>::Response, crate::Error>
        where
            C: MpdCmd,
    {
        self.exec_generic(cmd).await
    }

     */

    pub async fn exec_enumresp<C>(&mut self, cmd: C) -> Result<EnumResponse, crate::Error>
    where
        C: MpdCmd,
    {
        self.exec(cmd).await.map(Into::into)
    }

    pub async fn exec<C>(
        &mut self,
        cmd: C,
    ) -> Result<<C::Handler as ResponseHandler>::Response, crate::Error>
    where
        C: MpdCmd,
    {
        let cmdline = cmd.to_cmdline();

        self.send_command(&cmdline).await?;

        let br = self.stream.as_mut().ok_or(Error::Disconnected)?;

        // Handle the response associated with this command
        C::Handler::handle(br).await
    }

    async fn send_command(&mut self, line: &str) -> Result<(), crate::Error> {
        // Get the underlying TcpStream and write command to the socket
        self.stream
            .as_mut()
            .ok_or_else(|| crate::Error::Disconnected)?
            .get_mut()
            .write_all(line.as_bytes())
            .await
            .map_err(|_| crate::Error::Disconnected)?;

        Ok(())
    }
}
