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
use serde::Serialize;
use std::fmt::Debug;
use std::io;

#[derive(Serialize, Debug, Default)]
/// Mpd status response
pub struct Status {
    /// Name of current partition
    pub partition: String,
    /// Volume (0 - 100)
    pub volume: i32,
    /// Repeat
    pub repeat: bool,
    /// Random
    pub random: bool,
    /// 0, 1 or Oneshot
    pub single: String,
    /// Consume
    pub consume: bool,
    /// Playlist version number
    pub playlist: u32,
    pub playlistlength: i32,
    pub song: i32,
    pub songid: i32,
    pub nextsong: i32,
    pub nextsongid: i32,
    pub time: i32,
    pub elapsed: i32,
    pub duration: i32,
    pub mixrampdb: f32,
    /// mixrampdelay in seconds
    pub mixrampdelay: i32,
    pub state: String,
    /// Instantaneous bitrate in kbps
    pub bitrate: i32,
    /// crossfade in seconds
    pub xfade: i32,
    pub audio: String,
    pub updating_db: i32,
    pub error: String,
}

#[derive(Serialize, Clone, Debug, Default)]
/// Track in Queue
pub struct QueuedTrack {
    pub file: String,
    pub artist_sort: String,
    pub album_artist: String,
    pub album_artist_sort: String,
    pub performer: Vec<String>,
    pub title: String,
    pub track: i32,
    pub album: String,
    pub artist: String,
    pub pos: i32,
    pub id: i32,
}

#[derive(Serialize, Clone, Debug, Default)]
/// Mpd database statistics
pub struct Stats {
    pub uptime: i32,
    pub playtime: i32,
    pub artists: i32,
    pub albums: i32,
    pub songs: i32,
    pub db_playtime: i32,
    pub db_update: i32,
}

#[derive(Serialize, Debug)]
/// Subsystem
pub enum Subsystem {
    Database,
    Player,
    Mixer,
    Options,
    Update,
    StoredPlaylist,
    Playlist,
    Output,
    Partitions,
    Sticker,
    Subscription,
    Message,
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
        let lines = self.read_resp_lines().await?;
        let mut stats = Stats::default();

        for l in lines {
            if let Some((k, v)) = l.split(": ").next_tuple() {
                match k {
                    "uptime" => stats.uptime = v.parse().unwrap_or_default(),
                    "albums" => stats.albums = v.parse().unwrap_or_default(),
                    "artists" => stats.artists = v.parse().unwrap_or_default(),
                    "songs" => stats.songs = v.parse().unwrap_or_default(),
                    "db_playtime" => stats.db_playtime = v.parse().unwrap_or_default(),
                    "db_update" => stats.db_update = v.parse().unwrap_or_default(),
                    "playtime" => stats.playtime = v.parse().unwrap_or_default(),
                    _ => warn!("unknown field: {}: {}", k, v),
                }
            }
        }

        Ok(stats)
    }

    pub async fn status(&mut self) -> io::Result<Status> {
        self.send_cmd("status").await?;
        let lines = self.read_resp_lines().await?;
        let mut status = Status::default();

        for l in lines {
            if let Some((k, v)) = l.split(": ").next_tuple() {
                match k {
                    "partition" => status.partition = v.to_string(),
                    "single" => status.single = v.to_string(),
                    "state" => status.state = v.to_string(),
                    "volume" => status.volume = v.parse().unwrap_or_default(),
                    "repeat" => status.repeat = v.parse::<i32>().unwrap_or_default() != 0,
                    "random" => status.random = v.parse::<i32>().unwrap_or_default() != 0,
                    "consume" => status.consume = v.parse::<i32>().unwrap_or_default() != 0,
                    "playlistlength" => status.playlistlength = v.parse().unwrap_or_default(),
                    "playlist" => status.playlist = v.parse().unwrap_or_default(),
                    "song" => status.song = v.parse().unwrap_or_default(),
                    "songid" => status.songid = v.parse().unwrap_or_default(),
                    "nextsong" => status.nextsong = v.parse().unwrap_or_default(),
                    "nextsongid" => status.nextsongid = v.parse().unwrap_or_default(),
                    "elapsed" => status.elapsed = v.parse().unwrap_or_default(),
                    "duration" => status.duration = v.parse().unwrap_or_default(),
                    "time" => status.time = v.parse().unwrap_or_default(),
                    "mixrampdb" => status.mixrampdb = v.parse().unwrap_or_default(),
                    "audio" => status.audio = v.to_string(),
                    "bitrate" => status.bitrate = v.parse().unwrap_or_default(),
                    "updating_db" => status.updating_db = v.parse().unwrap_or_default(),
                    _ => warn!("Unknown status field: {}: {}", k, v),
                }
            }
        }

        Ok(status)
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
        let mut lines = self.read_resp_lines().await?;

        if lines.len() != 1 {
            log::warn!("More than one line");
        }

        let line = lines.pop().unwrap_or_default();

        if let Some((k, v)) = line.split(": ").next_tuple() {
            if k != "changed" {
                log::warn!("k not changed");
                return Ok(None);
            }

            return Ok(match v {
                "database" => Some(Subsystem::Database),
                "player" => Some(Subsystem::Player),
                "mixer" => Some(Subsystem::Mixer),
                "options" => Some(Subsystem::Options),
                "update" => Some(Subsystem::Update),
                "stored_playlist" => Some(Subsystem::StoredPlaylist),
                "playlist" => Some(Subsystem::Playlist),
                "output" => Some(Subsystem::Output),
                "partitions" => Some(Subsystem::Partitions),
                "sticker" => Some(Subsystem::Sticker),
                "subscription" => Some(Subsystem::Subscription),
                "message" => Some(Subsystem::Message),
                _ => {
                    log::debug!("unknown subsystem {}", v);
                    None
                }
            });
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
            .read_resp_lines()
            .await?
            .iter()
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
        let lines = self.read_resp_lines().await?;

        Ok(lines
            .into_iter()
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
        let lines = self.read_resp_lines().await?;

        let mut qi = QueuedTrack::default();
        let mut vec = Vec::new();

        for line in lines {
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
    async fn read_resp_lines(&mut self) -> io::Result<Vec<String>> {
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

        Ok(v)
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
