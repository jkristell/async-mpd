use async_net::{AsyncToSocketAddrs, TcpStream};
use futures::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use itertools::Itertools;
use log::info;

use std::{io, net::SocketAddr};

use crate::{
    filter::Filter,
    response::{self, Mixed},
    Stats, Status, Subsystem, Track,
};

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
        self.version = self.read_resp_line().await?;
        info!("version: {}", self.version);
        Ok(())
    }

    /// Get stats on the music database
    pub async fn stats(&mut self) -> Result<Stats, Error> {
        self.cmd("stats").await?;
        let lines = self.read_resp().await?;
        serde_yaml::from_str(&lines).map_err(|err| Error::ResponseError {
            reply: lines,
            errmsg: err.to_string(),
        })
    }

    pub async fn status(&mut self) -> Result<Status, Error> {
        self.cmd("status").await?;
        let lines = self.read_resp().await?;
        serde_yaml::from_str(&lines).map_err(|err| Error::ResponseError {
            reply: lines,
            errmsg: err.to_string(),
        })
    }

    pub async fn update(&mut self, path: Option<&str>) -> Result<i32, Error> {
        self.cmd(Cmd::new("update", path)).await?;
        let r = self.read_resp_line().await?;

        let db_version = match r.split(": ").next_tuple() {
            Some(("updating_db", dbv)) => dbv.parse().unwrap_or(0),
            _ => 0,
        };

        Ok(db_version)
    }

    pub async fn rescan(&mut self, path: Option<&str>) -> Result<i32, Error> {
        self.cmd(Cmd::new("rescan", path)).await?;
        let r = self.read_resp_line().await?;

        let db_version = match r.split(": ").next_tuple() {
            Some(("updating_db", dbv)) => dbv.parse().unwrap_or(0),
            _ => 0,
        };

        Ok(db_version)
    }

    pub async fn idle(&mut self) -> Result<Option<Subsystem>, Error> {
        self.cmd("idle").await?;
        let resp = self.read_resp().await?;
        let mut lines = resp.lines();

        let line = lines.next().unwrap_or_default();
        for unexpected_line in lines {
            log::warn!("More than one line in idle response: {}", unexpected_line);
        }

        if let Some((k, v)) = line.splitn(2, ": ").next_tuple() {
            if k != "changed" {
                log::warn!("k not changed");
                return Ok(None);
            }

            return Ok(serde_yaml::from_str(v).ok());
        }
        Ok(None)
    }

    pub async fn noidle(&mut self) -> Result<(), Error> {
        self.cmd("noidle").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn setvol(&mut self, volume: u32) -> Result<(), Error> {
        self.cmd(Cmd::new("setvol", Some(volume))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn repeat(&mut self, repeat: bool) -> Result<(), Error> {
        let repeat = if repeat { 1 } else { 0 };
        self.cmd(Cmd::new("repeat", Some(repeat))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn random(&mut self, random: bool) -> Result<(), Error> {
        let random = if random { 1 } else { 0 };
        self.cmd(Cmd::new("random", Some(random))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn consume(&mut self, consume: bool) -> Result<(), Error> {
        let consume = if consume { 1 } else { 0 };
        self.cmd(Cmd::new("consume", Some(consume))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    // Playback controls

    pub async fn play(&mut self) -> Result<(), Error> {
        self.play_pause(true).await
    }

    pub async fn playid(&mut self, id: u32) -> Result<(), Error> {
        self.cmd(Cmd::new("playid", Some(id))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), Error> {
        self.play_pause(false).await
    }

    pub async fn play_pause(&mut self, play: bool) -> Result<(), Error> {
        let play = if play { 0 } else { 1 };
        self.cmd(Cmd::new("pause", Some(play))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<(), Error> {
        self.cmd("next").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn prev(&mut self) -> Result<(), Error> {
        self.cmd("prev").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.cmd("stop").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    // Music database commands

    pub async fn listall(&mut self, path: Option<String>) -> Result<Vec<String>, Error> {
        self.cmd(Cmd::new("listall", path)).await?;

        Ok(self
            .read_resp()
            .await?
            .lines()
            .filter_map(|line| {
                if line.starts_with("file: ") {
                    Some(line[6..].to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn listallinfo(&mut self, path: Option<&str>) -> Result<Vec<Mixed>, Error> {
        self.cmd(Cmd::new("listallinfo", path)).await?;

        let resp = self.read_resp().await?;
        let r = response::mixed(&resp);
        Ok(r)
    }

    // Queue handling commands

    pub async fn queue_add(&mut self, path: &str) -> Result<(), Error> {
        self.cmd(Cmd::new("add", Some(path))).await?;
        self.read_ok_resp().await
    }

    pub async fn queue_clear(&mut self) -> Result<(), Error> {
        self.cmd("clear").await?;
        self.read_ok_resp().await
    }

    pub async fn queue(&mut self) -> Result<Vec<Track>, Error> {
        self.cmd("playlistinfo").await?;
        let resp = self.read_resp().await?;
        let vec = response::tracks(&resp);
        Ok(vec)
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
        self.cmd(Cmd::new("search", filter.to_query())).await?;
        let resp = self.read_resp().await?;
        let tracks = response::tracks(&resp);
        Ok(tracks)
    }

    async fn cmd(&mut self, cmd: impl Into<Cmd>) -> io::Result<()> {
        let r = cmd.into().to_string();
        self.bufreader.get_mut().write_all(r.as_bytes()).await?;
        Ok(())
    }

    /// Read all response lines
    async fn read_resp(&mut self) -> Result<String, Error> {
        let mut v = Vec::new();

        loop {
            let mut line = String::new();

            if self.bufreader.read_line(&mut line).await? == 0 {
                return Err(Error::Disconnected);
            }

            let line = line.trim();

            if line == "OK" {
                break;
            }

            if line.starts_with("ACK ") {
                log::trace!("Cmd error: {}", line);
                return Err(Error::CommandError { msg: line.into() });
            }

            v.push(line.to_string())
        }

        Ok(v.join("\n"))
    }

    /// Expect one line response
    async fn read_resp_line(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        self.bufreader.read_line(&mut line).await?;
        Ok(line.trim().to_string())
    }

    /// Read and expect OK response line
    async fn read_ok_resp(&mut self) -> Result<(), Error> {
        let mut line = String::new();
        self.bufreader.read_line(&mut line).await?;

        if &line != "OK\n" {
            return Err(Error::ResponseError {
                reply: line.to_string(),
                errmsg: "Expected OK".to_string(),
            });
        }

        Ok(())
    }
}

struct Cmd {
    cmd: &'static str,
    arg: Option<String>,
}

impl Cmd {
    fn new<T: ToString>(cmd: &'static str, arg: Option<T>) -> Self {
        Self {
            cmd,
            arg: arg.map(|a| a.to_string()),
        }
    }

    fn to_string(&self) -> String {
        if let Some(arg) = &self.arg {
            format!("{} \"{}\"\n", self.cmd, arg.to_string())
        } else {
            format!("{}\n", self.cmd)
        }
    }
}

impl From<&'static str> for Cmd {
    fn from(cmd: &'static str) -> Self {
        Self { cmd, arg: None }
    }
}
