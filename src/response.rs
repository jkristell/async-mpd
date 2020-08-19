use itertools::Itertools;
use log::warn;
use serde::Serialize;
use std::time::Duration;

use crate::{Directory, Playlist, Track};

#[derive(Serialize, Debug)]
/// Response from commands that returns entries with metadata and tags
pub enum Mixed {
    File(Track),
    Directory(Directory),
    Playlist(Playlist),
}

impl Mixed {
    /// Try to convert to Track
    pub fn track(&self) -> Option<&Track> {
        match self {
            Mixed::File(t) => Some(t),
            _ => None,
        }
    }
    /// Try to convert to Directory
    pub fn directory(&self) -> Option<&Directory> {
        match self {
            Mixed::Directory(d) => Some(d),
            _ => None,
        }
    }
}

pub(crate) fn tracks(resp: &str) -> Vec<Track> {
    mixed(resp)
        .iter()
        .filter_map(Mixed::track)
        .cloned()
        .collect()
}

pub(crate) fn mixed(resp: &str) -> Vec<Mixed> {
    let mut ms = Vec::new();
    let mut cur: Option<Mixed> = None;

    // TODO: replace this with serde deserialization

    for line in resp.lines() {
        if let Some((k, v)) = line.splitn(2, ": ").next_tuple() {
            match k {
                "directory" => {
                    if let Some(old) = cur.replace(Mixed::Directory(Directory::default())) {
                        ms.push(old);
                    }
                }
                "file" => {
                    if let Some(old) = cur.replace(Mixed::File(Track::default())) {
                        ms.push(old)
                    }
                }
                "playlist" => {
                    if let Some(item) = cur.replace(Mixed::Playlist(Playlist::default())) {
                        ms.push(item)
                    }
                }
                _ => {}
            }

            match cur.as_mut() {
                Some(Mixed::File(t)) => fill_track(t, k, v),
                Some(Mixed::Directory(d)) => fill_directory(d, k, v),
                Some(Mixed::Playlist(p)) => fill_playlist(p, k, v),
                _ => log::warn!("No currunt fdp set"),
            }
        }
    }

    // Special case for the last result
    if let Some(item) = cur {
        ms.push(item);
    }

    ms
}

fn fill_directory(d: &mut Directory, k: &str, v: &str) {
    match k {
        "directory" => d.path = v.to_string(),
        "Last-Modified" => d.last_modified = v.parse().ok(),
        _ => log::warn!("Unhandled directory key: {}", k),
    }
}

fn fill_playlist(p: &mut Playlist, k: &str, v: &str) {
    match k {
        "playlist" => p.path = v.to_string(),
        _ => log::warn!("Unhandled playlist key: {}", k),
    }
}

fn fill_track(t: &mut Track, k: &str, v: &str) {
    match k {
        "file" => t.file = v.to_string(),
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
        }
        "Format" => t.format = Some(v.to_string()),
        "Time" => t.time = v.parse().ok(),
        "Date" => t.date = Some(v.to_string()),
        "Disc" => t.disc = v.parse().ok(),
        "Label" => t.label = Some(v.to_string()),

        "duration" => {
            t.duration = Duration::from_secs_f64(v.parse().unwrap());
            log::debug!("v: {} {} {:?}", t.file, v, t.duration);
        }

        "MUSICBRAINZ_ARTISTID" => t.musicbrainz_artistid = Some(v.to_string()),
        "MUSICBRAINZ_ALBUMID" => t.musicbrainz_albumid = Some(v.to_string()),
        "MUSICBRAINZ_TRACKID" => t.musicbraiz_trackid = Some(v.to_string()),
        "MUSICBRAINZ_ALBUMARTISTID" => t.musicbrainz_albumartistid = Some(v.to_string()),
        "MUSICBRAINZ_RELEASETRACKID" => t.musicbraiz_releasetrackid = Some(v.to_string()),
        "Composer" => t.composer = Some(v.to_string()),

        _ => warn!("Unhandled track tag: {}: {}", k, v),
    }
}
