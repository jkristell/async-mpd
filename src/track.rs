use itertools::Itertools;
use log::warn;
use serde::Serialize;

use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Serialize, Clone, Debug, Default)]
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

pub(crate) fn from_response(resp: &str) -> Vec<Track> {
    let mut t = Track::default();
    let mut tracks = Vec::new();

    // TODO: replace this with serde deserialization
    for line in resp.lines() {

        log::debug!("l: {}", line);
        if let Some((k, v)) = line.splitn(2, ": ").next_tuple() {
            match k {
                "file" | "directory" | "playlist" => {
                    if !t.file.is_empty() {
                        tracks.push(t.clone());
                        t = Track::default();
                    }
                    t.file = v.to_string();
                }
                "Title" => t.title = Some(v.to_string()),
                "Genre" => t.genre = Some(v.to_string()),
                "Track" => t.track = v.parse().ok(),
                "Album" => t.album = Some(v.to_string()),
                "AlbumSort" => t.album_sort = Some(v.to_string()),
                "Artist" => t.artist = Some(v.to_string()),
                "ArtistSort" => t.artist_sort = Some(v.to_string()),
                "AlbumArtist" => t.album_artist = Some(v.to_string()),
                "AlbumArtistSort" => t.album_artist_sort = Some(v.to_string()),
                "Pos" => t.pos = v.parse().ok(),
                "Id" => t.id = v.parse().ok(),
                "Performer" => t.performer.push(v.to_string()),
                "Last-Modified" => t.last_modified = v.parse().ok(),
                "OriginalDate" => {
                    t.original_date = Some(v.to_string());
                },
                "Format" => t.format = Some(v.to_string()),
                "Time" => t.time = v.parse().ok(),
                "Date" => t.date = Some(v.to_string()),
                "Disc" => t.disc = v.parse().ok(),
                "Label" => t.label = Some(v.to_string()),

                "duration" => {
                    t.duration = Duration::from_secs_f64( v.parse().unwrap());
                    log::debug!("v: {} {} {:?}", t.file, v, t.duration);
                },

                "MUSICBRAINZ_ARTISTID" => t.musicbrainz_artistid = Some(v.to_string()),
                "MUSICBRAINZ_ALBUMID" => t.musicbrainz_albumid = Some(v.to_string()),
                "MUSICBRAINZ_TRACKID" => t.musicbraiz_trackid = Some(v.to_string()),
                "MUSICBRAINZ_ALBUMARTISTID" => t.musicbrainz_albumartistid = Some(v.to_string()),
                "MUSICBRAINZ_RELEASETRACKID" => t.musicbraiz_releasetrackid = Some(v.to_string()),
                "Composer" => t.composer = Some(v.to_string()),

                _ => warn!("Unhandled track tag: {}: {}", k, v),
            }
        }
    }

    if !t.file.is_empty() {
        tracks.push(t);
    }

    tracks
}
