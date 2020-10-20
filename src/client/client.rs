use async_net::{AsyncToSocketAddrs, TcpStream};
use futures_lite::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use itertools::Itertools;

use std::str::FromStr;
use std::{io, net::SocketAddr};

use crate::client::respmap::RespMap;
use crate::{
    client::responses::{self, MixedResponse},
    client::Filter,
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
        self.version = self.read_resp_line().await?;
        log::debug!("Connected: {}", self.version);
        Ok(())
    }

    /// Get stats on the music database
    pub async fn stats(&mut self) -> Result<Stats, Error> {
        self.cmd("stats").await?;
        let lines = self.read_resp().await?;

        let map = RespMap::from_string(lines);
        Ok(map.into())
    }

    pub async fn status(&mut self) -> Result<Status, Error> {
        self.cmd("status").await?;
        let lines = self.read_resp().await?;

        let map = RespMap::from_string(lines);
        Ok(map.into())
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

    pub async fn idle(&mut self) -> Result<Subsystem, Error> {
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
                return Err(Error::CommandError {
                    msg: "".to_string(),
                });
            }

            let subsystem = Subsystem::from_str(v)?;
            return Ok(subsystem);
        }
        Err(Error::CommandError {
            msg: "".to_string(),
        })
    }

    pub async fn noidle(&mut self) -> Result<(), Error> {
        self.okcmd("noidle").await
    }

    pub async fn setvol(&mut self, volume: u32) -> Result<(), Error> {
        self.okcmd(Cmd::new("setvol", Some(volume))).await
    }

    pub async fn repeat(&mut self, repeat: bool) -> Result<(), Error> {
        let repeat: i32 = repeat.into();
        self.okcmd(Cmd::new("repeat", Some(repeat))).await
    }

    pub async fn random(&mut self, random: bool) -> Result<(), Error> {
        let random: i32 = random.into();
        self.okcmd(Cmd::new("random", Some(random))).await
    }

    pub async fn consume(&mut self, consume: bool) -> Result<(), Error> {
        let consume: i32 = consume.into();
        self.okcmd(Cmd::new("consume", Some(consume))).await
    }

    // Playback controls

    pub async fn play(&mut self) -> Result<(), Error> {
        self.play_pause(true).await
    }

    pub async fn playid(&mut self, id: u32) -> Result<(), Error> {
        self.okcmd(Cmd::new("playid", Some(id))).await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), Error> {
        self.play_pause(false).await
    }

    pub async fn play_pause(&mut self, play: bool) -> Result<(), Error> {
        let play: i32 = (!play).into();
        self.okcmd(Cmd::new("pause", Some(play))).await
    }

    pub async fn next(&mut self) -> Result<(), Error> {
        self.okcmd("next").await
    }

    pub async fn prev(&mut self) -> Result<(), Error> {
        self.okcmd("prev").await
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.okcmd("stop").await
    }

    //
    // Music database commands
    //

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

    pub async fn listallinfo(&mut self, path: Option<&str>) -> Result<Vec<MixedResponse>, Error> {
        self.cmd(Cmd::new("listallinfo", path)).await?;
        Ok(responses::mixed_stream(&mut self.bufreader).await?)
    }

    // Queue handling commands

    pub async fn queue_add(&mut self, path: &str) -> Result<(), Error> {
        self.okcmd(Cmd::new("add", Some(path))).await
    }

    pub async fn queue_clear(&mut self) -> Result<(), Error> {
        self.okcmd("clear").await
    }

    pub async fn queue(&mut self) -> Result<Vec<Track>, Error> {
        self.cmd("playlistinfo").await?;
        Ok(responses::tracks(&mut self.bufreader).await?)
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
        let tracks = responses::tracks(&mut self.bufreader).await?;
        Ok(tracks)
    }

    async fn cmd(&mut self, cmd: impl Into<Cmd>) -> io::Result<()> {
        let r = cmd.into().to_string();
        self.bufreader.get_mut().write_all(r.as_bytes()).await?;
        Ok(())
    }

    async fn okcmd(&mut self, cmd: impl Into<Cmd>) -> Result<(), Error> {
        let r = cmd.into().to_string();
        log::debug!("cmd: {}", r);
        self.bufreader.get_mut().write_all(r.as_bytes()).await?;
        self.read_ok_resp().await?;
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
