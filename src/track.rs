use itertools::Itertools;
use log::warn;
use serde::Serialize;

#[derive(Serialize, Clone, Debug, Default)]
/// Track
pub struct Track {
    pub file: String,
    pub artist_sort: String,
    pub album_artist: String,
    pub album_artist_sort: String,
    pub performer: Vec<String>,
    pub title: String,
    pub track: u32,
    pub album: String,
    pub artist: String,
    pub pos: Option<u32>,
    pub id: u32,
    pub last_modified: String,
    pub format: String,
    pub time: u32,
    pub duration: u32,
    pub label: String,
    pub date: String,
    pub original_date: String,
    pub disc: u32,
    pub musicbraiz_trackid: String,
    pub musicbrainz_albumid: String,
    pub musicbrainz_albumartistid: String,
    pub musicbrainz_artistid: String,
}

pub(crate) fn from_lines(resp: &str) -> Vec<Track> {
    let mut t = Track::default();
    let mut tracks = Vec::new();

    // TODO: replace this with serde deserialization
    for line in resp.lines() {
        if let Some((k, v)) = line.splitn(2, ": ").next_tuple() {
            match k {
                "file" => {
                    if !t.file.is_empty() {
                        tracks.push(t.clone());
                        t = Track::default();
                    }
                    t.file = v.to_string();
                }
                "Title" => t.title = v.to_string(),
                "Track" => t.track = v.parse().unwrap_or_default(),
                "Album" => t.album = v.to_string(),
                "Artist" => t.artist = v.to_string(),
                "ArtistSort" => t.artist_sort = v.to_string(),
                "AlbumArtist" => t.album_artist = v.to_string(),
                "AlbumArtistSort" => t.album_artist_sort = v.to_string(),
                "Pos" => t.pos = v.parse().ok(),
                "Id" => t.id = v.parse().unwrap_or_default(),
                "Performer" => t.performer.push(v.to_string()),
                "Last-Modified" => t.last_modified = v.to_string(),
                "Format" => t.format = v.to_string(),
                "Time" => t.time = v.parse().unwrap_or_default(),
                "Date" => t.date = v.to_string(),
                "OriginalDate" => t.original_date = v.to_string(),
                "Disc" => t.disc = v.parse().unwrap_or_default(),
                "Label" => t.label = v.to_string(),
                "duration" => t.duration = v.parse().unwrap_or_default(),
                "MUSICBRAINZ_ARTISTID" => t.musicbrainz_artistid = v.to_string(),
                "MUSICBRAINZ_ALBUMID" => t.musicbrainz_albumid = v.to_string(),
                "MUSICBRAINZ_TRACKID" => t.musicbraiz_trackid = v.to_string(),
                "MUSICBRAINZ_ALBUMARTISTID" => t.musicbrainz_albumartistid = v.to_string(),

                _ => warn!("Unhandled qi field: {}: {}", k, v),
            }
        }
    }

    if !t.file.is_empty() {
        tracks.push(t);
    }

    tracks
}
