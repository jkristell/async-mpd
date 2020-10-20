use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use smart_default::SmartDefault;

#[derive(Deserialize, Serialize, Debug, Default)]
/// Playlist on the server
pub struct Playlist {
    pub path: String,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
/// Directory on the server
pub struct Directory {
    pub path: String,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
/// Mpd status response
pub struct Status {
    /// Name of current partition
    pub partition: Option<String>,
    /// Volume (0 - 100)
    pub volume: Option<u8>,
    pub repeat: bool,
    pub random: bool,
    /// 0, 1 or Oneshot
    pub single: String,
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
    #[serde(default)]
    pub elapsed: Option<Duration>,
    #[serde(default)]
    pub duration: Option<Duration>,
    pub mixrampdb: f32,
    /// mixrampdelay in seconds
    pub mixrampdelay: Option<u32>,
    /// Player status
    pub state: State,
    /// Instantaneous bitrate in kbps
    pub bitrate: Option<u16>,
    /// crossfade in seconds
    pub xfade: Option<u32>,
    pub audio: Option<String>,
    pub updating_db: Option<u32>,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, SmartDefault)]
/// Player status
pub enum State {
    Play,
    #[default]
    Stop,
    Pause,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
/// Mpd database statistics
pub struct Stats {
    pub uptime: Duration,
    pub playtime: Duration,
    pub artists: u32,
    pub albums: u32,
    pub songs: u32,
    pub db_playtime: Duration,
    pub db_update: i32,
}

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
    pub musicbraiz_workid: Option<String>,
    pub composer: Vec<String>,
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
