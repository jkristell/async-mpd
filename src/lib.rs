//! # Async-mpd
//!
//! Async-std based mpd client library for Rust
//!
//! ## Example:
//! ```
//! use async_mpd::MpdClient;
//!
//! #[async_std::main]
//! async fn main() -> std::io::Result<()> {
//!     // Connect to server
//!     let mut mpd = MpdClient::new("localhost:6600").await?;
//!
//!     // Get all tracks in the play queue and display them
//!     let queue = mpd.queue().await?;
//!     for track in queue {
//!         println!("{} - {}", track.artist, track.title);
//!     }
//!
//!     // Play track nr 2 in the queue
//!     mpd.playid(2).await?;
//!
//!     // Get and print the current server status
//!     println!("{:?}", mpd.status().await?);
//!
//!     // Set the volume to 50%
//!     mpd.setvol(50).await?;
//!
//!     Ok(())
//! }
//! ```

use async_std::{
    io::BufReader,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    prelude::*,
};
use itertools::Itertools;
use log::{info, trace, warn};
use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;
use std::io;
use std::time::Duration;

#[derive(Deserialize, Serialize, Debug, Default)]
/// Mpd status response
pub struct Status {
    /// Name of current partition
    pub partition: Option<String>,
    /// Volume (0 - 100)
    pub volume: Option<u8>,
    #[serde(deserialize_with = "de_bint")]
    pub repeat: bool,
    #[serde(deserialize_with = "de_bint")]
    pub random: bool,
    /// 0, 1 or Oneshot
    pub single: String,
    #[serde(deserialize_with = "de_bint")]
    pub consume: bool,
    /// Playlist version number
    pub playlist: u32,
    pub playlistlength: u32,
    pub song: Option<u32>,
    pub songid: Option<u32>,
    pub nextsong: Option<u32>,
    pub nextsongid: Option<u32>,
    // TODO: mpd returns this as "291:336" for 291.336 seconds.
    // It’s almost usually just a few ms ahead of elapsed,
    // so I’m not sure if we need this at all.
    pub time: Option<String>,
    #[serde(deserialize_with = "de_time_float")]
    #[serde(default)]
    pub elapsed: Option<Duration>,
    #[serde(deserialize_with = "de_time_float")]
    #[serde(default)]
    pub duration: Option<Duration>,
    pub mixrampdb: f32,
    /// mixrampdelay in seconds
    pub mixrampdelay: Option<u32>,
    /// TODO: make this an enum
    pub state: String,
    /// Instantaneous bitrate in kbps
    pub bitrate: Option<u16>,
    /// crossfade in seconds
    pub xfade: Option<u32>,
    pub audio: Option<String>,
    pub updating_db: Option<u32>,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
/// Track in Queue
pub struct QueuedTrack {
    pub file: String,
    pub artist_sort: String,
    pub album_artist: String,
    pub album_artist_sort: String,
    pub performer: Vec<String>,
    pub title: String,
    pub track: u32,
    pub album: String,
    pub artist: String,
    pub pos: u32,
    pub id: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
/// Mpd database statistics
pub struct Stats {
    #[serde(deserialize_with = "de_time_int")]
    pub uptime: Duration,
    #[serde(deserialize_with = "de_time_int")]
    pub playtime: Duration,
    pub artists: u32,
    pub albums: u32,
    pub songs: u32,
    #[serde(deserialize_with = "de_time_int")]
    pub db_playtime: Duration,
    pub db_update: i32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
/// Subsystem
pub enum Subsystem {
    Database,
    Player,
    Mixer,
    Options,
    Update,
    #[serde(rename = "stored_playlist")]
    StoredPlaylist,
    Playlist,
    Output,
    Partitions,
    Sticker,
    Subscription,
    Message,
}

/// Deserialize time from an integer that represents the seconds.
/// mpd uses int for the database stats (e.g. total time played).
fn de_time_int<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Duration::from_secs)
}

/// Deserialize time from a float that represents the seconds.
/// mpd uses floats for the current status (e.g. time elapsed in song).
fn de_time_float<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    f64::deserialize(deserializer)
        .map(Duration::from_secs_f64)
        .map(Some)
}

/// mpd uses bints (0 or 1) to represent booleans,
/// so we need a special parser for those.
fn de_bint<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        n => Err(de::Error::invalid_value(
            Unexpected::Unsigned(n as u64),
            &"zero or one",
        )),
    }
}

/// Mpd Client
pub struct MpdClient {
    bufreader: BufReader<TcpStream>,
    version: String,
    address: SocketAddr,
}

impl MpdClient {
    /// Create a new MpdClient and connect to `addr`
    pub async fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;

        // Save if we need to reconnect later
        let address = stream.peer_addr()?;
        let bufreader = BufReader::new(stream);

        let mut s = Self {
            bufreader,
            version: String::new(),
            address,
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
        self.send_cmd("stats").await?;
        let lines = self.read_resp().await?;
        Ok(serde_yaml::from_str(&lines)
            .unwrap_or_else(|e| panic!("Failed to parse mpd response: “{}” with {}", &lines, e)))
    }

    pub async fn status(&mut self) -> io::Result<Status> {
        self.send_cmd("status").await?;
        let lines = self.read_resp().await?;
        Ok(serde_yaml::from_str(&lines)
            .unwrap_or_else(|e| panic!("Failed to parse mpd response: “{}” with {}", &lines, e)))
    }

    pub async fn update(&mut self, path: Option<&str>) -> io::Result<i32> {
        self.send_cmd_with_arg("update", path).await?;
        let r = self.read_resp_line().await?;

        let db_version = match r.split(": ").next_tuple() {
            Some(("updating_db", dbv)) => dbv.parse().unwrap_or(0),
            _ => 0,
        };

        Ok(db_version)
    }

    pub async fn rescan(&mut self, path: Option<&str>) -> io::Result<i32> {
        self.send_cmd_with_arg("rescan", path).await?;
        let r = self.read_resp_line().await?;

        let db_version = match r.split(": ").next_tuple() {
            Some(("updating_db", dbv)) => dbv.parse().unwrap_or(0),
            _ => 0,
        };

        Ok(db_version)
    }

    pub async fn idle(&mut self) -> io::Result<Option<Subsystem>> {
        self.send_cmd("idle").await?;
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
        self.send_cmd("noidle").await?;
        Ok(())
    }

    pub async fn setvol(&mut self, volume: i32) -> io::Result<()> {
        self.send_cmd_with_arg("setvol", Some(volume)).await?;
        Ok(())
    }

    pub async fn repeat(&mut self, repeat: bool) -> io::Result<()> {
        let repeat = if repeat { 1 } else { 0 };
        self.send_cmd_with_arg("repeat", Some(repeat)).await?;
        Ok(())
    }

    pub async fn random(&mut self, random: bool) -> io::Result<()> {
        let random = if random { 1 } else { 0 };
        self.send_cmd_with_arg("random", Some(random)).await?;
        Ok(())
    }

    pub async fn consume(&mut self, consume: bool) -> io::Result<()> {
        let consume = if consume { 1 } else { 0 };
        self.send_cmd_with_arg("consume", Some(consume)).await?;
        Ok(())
    }

    // Playback controls

    pub async fn play(&mut self) -> io::Result<()> {
        self.play_pause(true).await
    }

    pub async fn playid(&mut self, id: i32) -> io::Result<()> {
        self.send_cmd_with_arg("playid", Some(id)).await?;
        self.read_resp_ok().await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> io::Result<()> {
        self.play_pause(false).await
    }

    pub async fn play_pause(&mut self, play: bool) -> io::Result<()> {
        let play = if play { 0 } else { 1 };
        self.send_cmd_with_arg("pause", Some(play)).await?;
        self.read_resp_ok().await?;
        Ok(())
    }

    pub async fn next(&mut self) -> io::Result<()> {
        self.send_cmd("next").await?;
        self.read_resp_ok().await?;
        Ok(())
    }

    pub async fn prev(&mut self) -> io::Result<()> {
        self.send_cmd("prev").await?;
        self.read_resp_ok().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> io::Result<()> {
        self.send_cmd("stop").await?;
        self.read_resp_ok().await?;
        Ok(())
    }

    // Filesystem

    pub async fn listall(&mut self, path: Option<String>) -> io::Result<Vec<String>> {
        self.send_cmd_with_arg("listall", path).await?;

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

    pub async fn listalldirs(&mut self) -> io::Result<Vec<String>> {
        self.send_cmd("listall").await?;
        let lines = self.read_resp().await?;

        Ok(lines
            .lines()
            .filter(|s| s.starts_with("directory: "))
            .map(|s| s[11..].to_string())
            .collect())
    }

    // Queue

    pub async fn queue_add(&mut self, path: &str) -> io::Result<()> {
        self.send_cmd(&format!("add {}", path)).await?;
        self.read_resp_ok().await
    }

    pub async fn queue_clear(&mut self) -> io::Result<()> {
        self.send_cmd("clear").await?;
        self.read_resp_ok().await
    }

    pub async fn queue(&mut self) -> io::Result<Vec<QueuedTrack>> {
        self.send_cmd("playlistinfo").await?;
        let resp = self.read_resp().await?;

        let mut qi = QueuedTrack::default();
        let mut vec = Vec::new();

        // TODO: replace this with serde deserialization
        for line in resp.lines() {
            if let Some((k, v)) = line.split(": ").next_tuple() {
                match k {
                    "file" => {
                        if !qi.file.is_empty() {
                            vec.push(qi.clone());
                            qi = QueuedTrack::default();
                        }
                        qi.file = v.to_string();
                    }
                    "Title" => qi.title = v.to_string(),
                    "Track" => qi.track = v.parse().unwrap_or_default(),
                    "Album" => qi.album = v.to_string(),
                    "Artist" => qi.artist = v.to_string(),
                    "Pos" => qi.pos = v.parse().unwrap_or_default(),
                    "Id" => qi.id = v.parse().unwrap_or_default(),
                    "Performer" => qi.performer.push(v.to_string()),
                    _ => warn!("Unhandled qi field: {}: {}", k, v),
                }
            }
        }

        if !qi.file.is_empty() {
            vec.push(qi);
        }

        Ok(vec)
    }

    async fn send_cmd(&mut self, cmd: &str) -> io::Result<()> {
        trace!("send_command: {}", cmd);
        let s = format!("{}\n", cmd);
        self.send_cmd_ll(&s).await?;
        Ok(())
    }

    async fn send_cmd_with_arg<T: Debug + ToString>(
        &mut self,
        cmd: &str,
        arg: Option<T>,
    ) -> io::Result<()> {
        trace!("cmd_with_arg: {} {:?}", cmd, arg);

        let s = if let Some(arg) = arg {
            format!("{} \"{}\"\n", cmd, arg.to_string())
        } else {
            format!("{}\n", cmd)
        };

        self.send_cmd_ll(&s).await?;
        Ok(())
    }

    async fn send_cmd_ll(&mut self, cmd: &str) -> io::Result<()> {
        let res = self.bufreader.get_mut().write_all(cmd.as_bytes()).await;

        // It's possible that the server had forgotten about us.
        // Reconnect and reestablish connection

        if let Err(_) = res {
            log::trace!("Lost connection, reconnecting");

            // Try reconnect
            let stream = TcpStream::connect(self.address).await?;
            self.bufreader = BufReader::new(stream);
            self.read_version().await?;

            log::trace!("Reconnected, resending command");
            self.bufreader.get_mut().write_all(cmd.as_bytes()).await?;
        }

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
    async fn read_resp_ok(&mut self) -> io::Result<()> {
        let mut line = String::new();
        self.bufreader.read_line(&mut line).await?;

        if &line != "OK\n" {
            warn!("Expected OK, got: {}", line);
        }

        Ok(())
    }
}
