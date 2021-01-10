use crate::{MixedResponse, Track};

pub enum Command<'a> {
    Stats,
    Status,
    Update(Option<&'a str>),
    Rescan(Option<&'a str>),
    Idle,
    NoIdle,
    Repeat(bool),
    Random(bool),
    Consume(bool),

    PlayPaus(bool),
    Next,
    Prev,
    Stop,

    Listall(Option<String>),

    QueueAdd(&'a str),
    QueueClear,

    Search(Option<String>),

    SetVol(u32),
    PlayId(u32),
    ListallInfo(Option<&'a str>),
    PlaylistInfo,
}

pub enum CommandResponse {
    Stats(crate::Stats),
    Status(crate::Status),
    DbVersion(i32),
    Subsystem(crate::Subsystem),
    Ok,
    Mixed(Vec<MixedResponse>),
    Tracks(Vec<Track>),
    Paths(Vec<String>),
}

impl From<CommandResponse> for crate::Stats {
    fn from(r: CommandResponse) -> crate::Stats {
        match r {
            CommandResponse::Stats(stats) => stats,
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for crate::Status {
    fn from(r: CommandResponse) -> crate::Status {
        match r {
            CommandResponse::Status(status) => status,
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for crate::Subsystem {
    fn from(r: CommandResponse) -> crate::Subsystem {
        match r {
            CommandResponse::Subsystem(subsystem) => subsystem,
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for i32 {
    fn from(r: CommandResponse) -> i32 {
        match r {
            CommandResponse::DbVersion(version) => version,
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for Vec<String> {
    fn from(r: CommandResponse) -> Vec<String> {
        match r {
            CommandResponse::Paths(v) => v,
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for Vec<MixedResponse> {
    fn from(r: CommandResponse) -> Vec<MixedResponse> {
        match r {
            CommandResponse::Mixed(v) => v,
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for () {
    fn from(r: CommandResponse) -> () {
        match r {
            CommandResponse::Ok => (),
            _ => unreachable!(),
        }
    }
}

impl From<CommandResponse> for Vec<Track> {
    fn from(cr: CommandResponse) -> Vec<Track> {
        match cr {
            CommandResponse::Tracks(tracks) => tracks,
            _ => unreachable!(),
        }
    }
}

impl Command<'_> {
    pub fn line(&self) -> String {
        let cmdname = self.cmdname();

        let args = match self {
            Command::PlayId(id) | Command::SetVol(id) => Some(id.to_string()),
            Command::Consume(enable)
            | Command::Repeat(enable)
            | Command::Random(enable)
            | Command::PlayPaus(enable) => Some((*enable as i32).to_string()),

            Command::QueueAdd(path) => Some(path.to_string()),
            Command::ListallInfo(maybe) | Command::Update(maybe) | Command::Rescan(maybe) => {
                maybe.map(ToString::to_string)
            }

            Command::Search(maybe) | Command::Listall(maybe) => maybe.clone(),

            Command::PlaylistInfo
            | Command::Stats
            | Command::Status
            | Command::Idle
            | Command::NoIdle
            | Command::Next
            | Command::Prev
            | Command::Stop
            | Command::QueueClear => None,
        };

        if let Some(arg) = args {
            format!("{} \"{}\"\n", cmdname, arg)
        } else {
            format!("{}\n", cmdname)
        }
    }

    pub fn cmdname(&self) -> &'static str {
        match self {
            Command::Stats => "stats",
            Command::Status => "status",
            Command::Update(_) => "update",
            Command::Rescan(_) => "rescan",
            Command::Idle => "idle",
            Command::NoIdle => "noidle",
            Command::Repeat(_) => "repeat",
            Command::Random(_) => "random",
            Command::Consume(_) => "consume",
            Command::Next => "next",
            Command::Prev => "prev",
            Command::Stop => "stop",
            Command::PlayPaus(_) => "pause",

            Command::Search(_) => "search",

            Command::QueueAdd(_) => "add",
            Command::QueueClear => "clear",

            Command::SetVol(_) => "setvol",
            Command::PlayId(_) => "playid",
            Command::Listall(_) => "listall",
            Command::ListallInfo(_) => "listallinfo",
            Command::PlaylistInfo => "playlistinfo",
        }
    }
}
