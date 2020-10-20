use async_net::TcpStream;
use futures_lite::{io::AsyncBufReadExt, io::BufReader, StreamExt};
use itertools::Itertools;
use serde::Serialize;

use crate::client::respmap::RespMap;
use crate::{Directory, Playlist, Subsystem, Track, State};
use crate::{Stats, Status};
use std::str::FromStr;

impl FromStr for Subsystem {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = match s {
            "partitions" => Subsystem::Partitions,
            "player" => Subsystem::Player,
            "mixer" => Subsystem::Mixer,
            "options" => Subsystem::Options,
            "update" => Subsystem::Update,
            "storedplaylist" => Subsystem::StoredPlaylist,
            "output" => Subsystem::Output,
            _ => return Err(crate::Error::ValueError { msg: s.into() }),
        };
        Ok(r)
    }
}

impl FromStr for State {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status = match s {
            "play" => State::Play,
            "pause" => State::Pause,
            "stop" => State::Stop,
            _ => return Err(crate::Error::ValueError { msg: s.into()}),
        };
        Ok(status)
    }
}


#[derive(Serialize, Debug)]
/// Response from commands that returns entries with metadata and tags
pub enum MixedResponse {
    File(Track),
    Directory(Directory),
    Playlist(Playlist),
}

impl MixedResponse {
    /// Try to convert to Track
    pub fn track(&self) -> Option<&Track> {
        match self {
            MixedResponse::File(t) => Some(t),
            _ => None,
        }
    }
    /// Try to convert to Directory
    pub fn directory(&self) -> Option<&Directory> {
        match self {
            MixedResponse::Directory(d) => Some(d),
            _ => None,
        }
    }

    pub fn playlist(&self) -> Option<&Playlist> {
        if let MixedResponse::Playlist(playlist) = self {
            Some(playlist)
        } else {
            None
        }
    }
}

pub(crate) async fn tracks(stream: &mut BufReader<TcpStream>) -> std::io::Result<Vec<Track>> {
    Ok(mixed_stream(stream)
        .await?
        .iter()
        .filter_map(MixedResponse::track)
        .cloned()
        .collect())
}

impl From<RespMap> for Directory {
    fn from(mut map: RespMap) -> Self {
        let dir = Directory {
            path: map.get_def("directory"),
            last_modified: map.get("Last-Modified"),
        };

        if !map.is_empty() {
            log::warn!("Status map not empty: {:?}", map.inner);
        }

        dir
    }
}

impl From<RespMap> for Playlist {
    fn from(mut map: RespMap) -> Self {
        let playlist = Playlist {
            path: map.get_def("playlist"),
            last_modified: map.get("Last-Modified"),
        };

        if !map.is_empty() {
            log::warn!("Status map not empty: {:?}", map.inner);
        }

        playlist
    }
}

impl From<RespMap> for MixedResponse {
    fn from(map: RespMap) -> Self {
        if map.contains_key("directory") {
            MixedResponse::Directory(Directory::from(map))
        } else if map.contains_key("playlist") {
            MixedResponse::Playlist(Playlist::from(map))
        } else {
            MixedResponse::File(Track::from(map))
        }
    }
}

pub async fn mixed_stream(
    stream: &mut BufReader<TcpStream>,
) -> std::io::Result<Vec<MixedResponse>> {
    let mut resvec = Vec::new();
    let mut map = RespMap::new();
    let mut lines = stream.lines();

    while let Some(line) = lines.next().await {
        let line = line?;
        let line = line.trim();

        if line == "OK" {
            // We're done
            resvec.push(MixedResponse::from(map));
            break;
        }

        if !map.is_empty()
            && (line.starts_with("directory:")
                || line.starts_with("file:")
                || line.starts_with("playlist:"))
        {
            // Add the previous record to the result vec
            resvec.push(MixedResponse::from(map));
            // Open a new record
            map = RespMap::new();
        }

        if let Some((k, v)) = line.splitn(2, ": ").next_tuple() {
            map.insert(k, v);
        }
    }

    Ok(resvec)
}

impl From<RespMap> for Track {
    fn from(mut map: RespMap) -> Self {
        let track = Track {
            file: map.get_def("file"),
            artist_sort: map.get("ArtistSort"),
            album_artist: map.get("AlbumArtist"),
            album_sort: map.get("AlbumSort"),
            album_artist_sort: map.get("AlbumArtistSort"),
            performer: map.get_vec("Performer"),
            genre: map.get("Genre"),
            title: map.get("Title"),
            track: map.get("Track"),
            album: map.get("Album"),
            artist: map.get("Artist"),
            pos: map.get("Pos"),
            id: map.get("Id"),
            last_modified: map.get("Last-Modified"),
            original_date: map.get("OriginalDate"),
            time: map.get("Time"),
            format: map.get("Format"),
            duration: map.as_duration_def("duration"),
            label: map.get("Label"),
            date: map.get("Date"),
            disc: map.get("Disc"),
            musicbraiz_trackid: map.get("MUSICBRAINZ_TRACKID"),
            musicbrainz_albumid: map.get("MUSICBRAINZ_ALBUMID"),
            musicbrainz_albumartistid: map.get("MUSICBRAINZ_ALBUMARTISTID"),
            musicbrainz_artistid: map.get("MUSICBRAINZ_ARTISTID"),
            musicbraiz_releasetrackid: map.get("MUSICBRAINZ_RELEASETRACKID"),
            musicbraiz_workid: map.get("MUSICBRAINZ_WORKID"),
            composer: map.get_vec("Composer"),
        };

        if !map.is_empty() {
            log::warn!("Track map not empty: {:?}", map.inner);
        }

        track
    }
}

impl From<RespMap> for Status {
    fn from(mut map: RespMap) -> Self {
        let status = Status {
            partition: map.get("partition"),
            volume: map.get("volume"),
            repeat: map.as_bool("repeat"),
            random: map.as_bool("random"),
            single: map.get_def("single"),
            consume: map.as_bool("consume"),
            playlist: map.get_def("playlist"),
            playlistlength: map.get_def("playlistlength"),
            song: map.get("song"),
            songid: map.get("songid"),
            nextsong: map.get("nextsong"),
            nextsongid: map.get("nextsongid"),
            time: map.get("time"),
            elapsed: map.as_duration("elapsed"),
            duration: map.as_duration("duration"),
            mixrampdb: map.get_def("mixrampdb"),
            mixrampdelay: map.get("mixrampdelay"),
            state: map.get_def("state"),
            bitrate: map.get("bitrate"),
            xfade: map.get("xfade"),
            audio: map.get("audio"),
            updating_db: map.get("updating_db"),
            error: map.get("error"),
        };

        if !map.is_empty() {
            log::warn!("Status map not empty: {:?}", map.inner);
        }

        status
    }
}

impl From<RespMap> for Stats {
    fn from(mut map: RespMap) -> Self {
        let stats = Stats {
            uptime: map.as_duration_def("uptime"),
            playtime: map.as_duration_def("playtime"),
            artists: map.get_def("artists"),
            albums: map.get_def("albums"),
            songs: map.get_def("songs"),
            db_playtime: map.as_duration_def("db_playtime"),
            db_update: map.get_def("db_update"),
        };

        if !map.is_empty() {
            log::warn!("Status map not empty: {:?}", map.inner);
        }
        stats
    }
}

#[cfg(test)]
mod test {
    use crate::client::respmap::RespMap;
    use crate::{State, Status};
    use std::time::Duration;

    #[test]
    fn parse_status() {
        let input = r#"\
volume: 50
repeat: 1
random: 1
single: 0
consume: 0
playlist: 2
playlistlength: 141
mixrampdb: 0.000000
state: play
song: 1
songid: 2
time: 149:308
elapsed: 149.029
bitrate: 878
duration: 307.760
audio: 44100:16:2
nextsong: 124
nextsongid: 125
"#;

        let reference = Status {
            partition: None,
            volume: Some(50),
            repeat: true,
            random: true,
            single: "0".into(),
            consume: false,
            playlist: 2,
            playlistlength: 141,
            song: Some(1),
            songid: Some(2),
            nextsong: Some(124),
            nextsongid: Some(125),
            time: Some("149:308".into()),
            elapsed: Some(Duration::from_secs_f64(149.029)),
            duration: Some(Duration::from_secs_f64(307.76)),
            mixrampdb: 0.0,
            mixrampdelay: None,
            state: State::Play,
            bitrate: Some(878),
            xfade: None,
            audio: Some("44100:16:2".into()),
            updating_db: None,
            error: None,
        };

        let parsed = Status::from(RespMap::from_string(input.into()));
        assert_eq!(parsed, reference);
    }
}
