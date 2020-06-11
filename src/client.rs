use async_std::{
    io::BufReader,
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};
use itertools::Itertools;
use log::{info, warn};
use std::io;

use crate::{
    filter::Filter,
    response::{self, Mixed},
    Stats, Status, Subsystem, Track,
};

/// Mpd Client
pub struct MpdClient {
    bufreader: BufReader<TcpStream>,
    version: String,
}

impl MpdClient {
    /// Create a new MpdClient and connect to `addr`
    pub async fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;

        // Save if we need to reconnect later
        let bufreader = BufReader::new(stream);

        let mut s = Self {
            bufreader,
            version: String::new(),
        };

        s.read_version().await?;

        Ok(s)
    }

    async fn read_version(&mut self) -> io::Result<()> {
        self.version = self.read_resp_line().await?;
        info!("version: {}", self.version);
        Ok(())
    }

    /// Get stats on the music database
    pub async fn stats(&mut self) -> io::Result<Stats> {
        self.cmd("stats").await?;
        let lines = self.read_resp().await?;
        Ok(serde_yaml::from_str(&lines)
            .unwrap_or_else(|e| panic!("Failed to parse mpd response: “{}” with {}", &lines, e)))
    }

    pub async fn status(&mut self) -> io::Result<Status> {
        self.cmd("status").await?;
        let lines = self.read_resp().await?;
        Ok(serde_yaml::from_str(&lines)
            .unwrap_or_else(|e| panic!("Failed to parse mpd response: “{}” with {}", &lines, e)))
    }

    pub async fn update(&mut self, path: Option<&str>) -> io::Result<i32> {
        self.cmd(Cmd::new("update", path)).await?;
        let r = self.read_resp_line().await?;

        let db_version = match r.split(": ").next_tuple() {
            Some(("updating_db", dbv)) => dbv.parse().unwrap_or(0),
            _ => 0,
        };

        Ok(db_version)
    }

    pub async fn rescan(&mut self, path: Option<&str>) -> io::Result<i32> {
        self.cmd(Cmd::new("rescan", path)).await?;
        let r = self.read_resp_line().await?;

        let db_version = match r.split(": ").next_tuple() {
            Some(("updating_db", dbv)) => dbv.parse().unwrap_or(0),
            _ => 0,
        };

        Ok(db_version)
    }

    pub async fn idle(&mut self) -> io::Result<Option<Subsystem>> {
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

    pub async fn noidle(&mut self) -> io::Result<()> {
        self.cmd("noidle").await?;
        Ok(())
    }

    pub async fn setvol(&mut self, volume: u32) -> io::Result<()> {
        self.cmd(Cmd::new("setvol", Some(volume))).await?;
        Ok(())
    }

    pub async fn repeat(&mut self, repeat: bool) -> io::Result<()> {
        let repeat = if repeat { 1 } else { 0 };
        self.cmd(Cmd::new("repeat", Some(repeat))).await?;
        Ok(())
    }

    pub async fn random(&mut self, random: bool) -> io::Result<()> {
        let random = if random { 1 } else { 0 };
        self.cmd(Cmd::new("random", Some(random))).await?;
        Ok(())
    }

    pub async fn consume(&mut self, consume: bool) -> io::Result<()> {
        let consume = if consume { 1 } else { 0 };
        self.cmd(Cmd::new("consume", Some(consume))).await?;
        Ok(())
    }

    // Playback controls

    pub async fn play(&mut self) -> io::Result<()> {
        self.play_pause(true).await
    }

    pub async fn playid(&mut self, id: u32) -> io::Result<()> {
        self.cmd(Cmd::new("playid", Some(id))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> io::Result<()> {
        self.play_pause(false).await
    }

    pub async fn play_pause(&mut self, play: bool) -> io::Result<()> {
        let play = if play { 0 } else { 1 };
        self.cmd(Cmd::new("pause", Some(play))).await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn next(&mut self) -> io::Result<()> {
        self.cmd("next").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn prev(&mut self) -> io::Result<()> {
        self.cmd("prev").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> io::Result<()> {
        self.cmd("stop").await?;
        self.read_ok_resp().await?;
        Ok(())
    }

    // Music database commands

    pub async fn listall(&mut self, path: Option<String>) -> io::Result<Vec<String>> {
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

    pub async fn listallinfo(&mut self, path: Option<&str>) -> io::Result<Vec<Mixed>> {
        self.cmd(Cmd::new("listallinfo", path)).await?;

        let resp = self.read_resp().await?;
        let r = response::mixed(&resp);
        Ok(r)
    }

    // Queue handling commands

    pub async fn queue_add(&mut self, path: &str) -> io::Result<()> {
        self.cmd(Cmd::new("add", Some(path))).await?;
        self.read_ok_resp().await
    }

    pub async fn queue_clear(&mut self) -> io::Result<()> {
        self.cmd("clear").await?;
        self.read_ok_resp().await
    }

    pub async fn queue(&mut self) -> io::Result<Vec<Track>> {
        self.cmd("playlistinfo").await?;
        let resp = self.read_resp().await?;
        let vec = response::tracks(&resp);
        Ok(vec)
    }

    /// # Example
    /// ```
    /// use async_mpd::{MpdClient, Tag, Filter, ToFilterExpr};
    ///
    /// #[async_std::main]
    /// async fn main() -> std::io::Result<()> {
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
    pub async fn search(&mut self, filter: &Filter) -> io::Result<Vec<Track>> {
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
    async fn read_resp(&mut self) -> io::Result<String> {
        let mut v = Vec::new();

        loop {
            let mut line = String::new();

            if self.bufreader.read_line(&mut line).await? == 0 {
                break;
            }

            let line = line.trim();

            if line == "OK" {
                break;
            }

            if line.starts_with("ACK ") {
                log::trace!("Cmd error: {}", line);
                break;
            }

            v.push(line.to_string())
        }

        Ok(v.join("\n"))
    }

    /// Expect one line response
    async fn read_resp_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        self.bufreader.read_line(&mut line).await?;
        Ok(line.trim().to_string())
    }

    /// Read and expect OK response line
    async fn read_ok_resp(&mut self) -> io::Result<()> {
        let mut line = String::new();
        self.bufreader.read_line(&mut line).await?;

        if &line != "OK\n" {
            warn!("Expected OK, got: {}", line);
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
