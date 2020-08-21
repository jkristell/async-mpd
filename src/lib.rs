//! # async-mpd
//!
//! Runtime agnostic mpd client library for Rust
//!
//! ## Example:
//! ```
//! use async_std as runtime;
//! use async_mpd::MpdClient;
//!
//! #[runtime::main]
//! async fn main() -> Result<(), async_mpd::Error> {
//!     // Connect to server
//!     let mut mpd = MpdClient::new("localhost:6600").await?;
//!
//!     // Get all tracks in the play queue and display them
//!     let queue = mpd.queue().await?;
//!     for track in queue {
//!         println!("{:?} - {:?}", track.artist, track.title);
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
//!
//! ## Using tokio:
//! ```
//! use tokio as runtime;
//! use async_mpd::MpdClient;
//!
//! #[runtime::main]
//! async fn main() -> Result<(), async_mpd::Error> {
//!     // Connect to server
//!     let mut mpd = MpdClient::new("localhost:6600").await?;
//!
//!     // Get all tracks in the play queue and display them
//!     let queue = mpd.queue().await?;
//!     for track in queue {
//!         println!("{:?} - {:?}", track.artist, track.title);
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

use chrono::{DateTime, Utc};
use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;
use std::time::Duration;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use {client::Error, client::MpdClient};

#[cfg(feature = "client")]
mod response;
#[cfg(feature = "client")]
pub use crate::response::Mixed;

mod filter;
pub use filter::{Filter, FilterExpr, ToFilterExpr};

#[derive(Deserialize, Serialize, Debug, Default)]
/// Playlist on the server
pub struct Playlist {
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
/// Directory on the server
pub struct Directory {
    pub path: String,
    pub last_modified: Option<DateTime<Utc>>,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default)]
/// Mpd status response
pub struct Status {
    /// Name of current partition
    pub partition: Option<String>,
    /// Volume (0 - 100)
    pub volume: Option<u8>,
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_bint")]
    pub repeat: bool,
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_bint")]
    pub random: bool,
    /// 0, 1 or Oneshot
    pub single: String,
    #[cfg(feature = "client")]
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
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_time_float")]
    #[serde(default)]
    pub elapsed: Option<Duration>,
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_time_float")]
    #[serde(default)]
    pub duration: Option<Duration>,
    pub mixrampdb: f32,
    /// mixrampdelay in seconds
    pub mixrampdelay: Option<u32>,
    //TODO: make this an enum
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
/// Mpd database statistics
pub struct Stats {
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_time_int")]
    pub uptime: Duration,
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_time_int")]
    pub playtime: Duration,
    pub artists: u32,
    pub albums: u32,
    pub songs: u32,
    #[cfg(feature = "client")]
    #[serde(deserialize_with = "de_time_int")]
    pub db_playtime: Duration,
    pub db_update: i32,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
/// Track
pub struct Track {
    pub file: String,
    pub artist_sort: Option<String>,
    pub album_artist: Option<String>,
    pub album_sort: Option<String>,
    pub album_artist_sort: Option<String>,
    pub performer: Vec<String>,
    pub genre: Option<String>,
    pub title: Option<String>,
    pub track: Option<u32>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub pos: Option<u32>,
    pub id: Option<u32>,
    pub last_modified: Option<DateTime<Utc>>,
    pub original_date: Option<String>,
    pub time: Option<String>,
    pub format: Option<String>,
    pub duration: Duration,
    pub label: Option<String>,
    pub date: Option<String>,
    pub disc: Option<u32>,
    pub musicbraiz_trackid: Option<String>,
    pub musicbrainz_albumid: Option<String>,
    pub musicbrainz_albumartistid: Option<String>,
    pub musicbrainz_artistid: Option<String>,
    pub musicbraiz_releasetrackid: Option<String>,
    pub composer: Option<String>,
}

#[derive(Copy, Clone, Debug)]
/// Track tags
pub enum Tag {
    Artist,
    ArtistSort,
    Album,
    AlbumSort,
    AlbumArtist,
    AlbumSortOrder,
    Title,
    Track,
    Name,
    Genre,
    Date,
    Composer,
    Performer,
    Conductor,
    Work,
    Grouping,
    Comment,
    Disc,
    Label,
    MusicbrainzArtistId,
    MusicbrainzAlbumId,
    MusicbrainzAlbumArtistId,
    MusicbrainzTrackId,
    MusicbrainzReleaseTrackId,
    MusicbrainzWorkId,
    Any,
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

#[cfg(feature = "client")]
/// Deserialize time from an integer that represents the seconds.
/// mpd uses int for the database stats (e.g. total time played).
fn de_time_int<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Duration::from_secs)
}

#[cfg(feature = "client")]
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

#[cfg(feature = "client")]
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
