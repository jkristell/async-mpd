use crate::client::resp::respmap_handlers::{ListallResponse, ListallinfoResponse};
use crate::protocol::Stats;
use crate::{protocol, DatabaseVersion, Error, Status, Subsystem, Track};
use async_net::TcpStream;
use futures_lite::io::BufReader;
use futures_lite::AsyncBufReadExt;

pub mod handlers;
pub mod respmap;
pub mod respmap_handlers;

/// Expect one line response
pub(crate) async fn read_resp_line(reader: &mut BufReader<TcpStream>) -> Result<String, Error> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    Ok(line.trim().to_string())
}

pub enum EnumResponse {
    Ok,
    ListAllInfo(ListallinfoResponse),
    Tracks(Vec<Track>),
    Listall(ListallResponse),
    Subsystem(Subsystem),
    DatabaseVersion(DatabaseVersion),
    Status(Status),
    Stats(Stats),
}

impl From<()> for EnumResponse {
    fn from(_: ()) -> Self {
        EnumResponse::Ok
    }
}

impl From<ListallinfoResponse> for EnumResponse {
    fn from(l: ListallinfoResponse) -> Self {
        EnumResponse::ListAllInfo(l)
    }
}

impl From<Vec<protocol::Track>> for EnumResponse {
    fn from(t: Vec<Track>) -> Self {
        EnumResponse::Tracks(t)
    }
}

impl From<ListallResponse> for EnumResponse {
    fn from(l: ListallResponse) -> Self {
        EnumResponse::Listall(l)
    }
}

impl From<protocol::Subsystem> for EnumResponse {
    fn from(s: Subsystem) -> Self {
        EnumResponse::Subsystem(s)
    }
}

impl From<protocol::DatabaseVersion> for EnumResponse {
    fn from(d: DatabaseVersion) -> Self {
        EnumResponse::DatabaseVersion(d)
    }
}

impl From<protocol::Status> for EnumResponse {
    fn from(s: Status) -> Self {
        EnumResponse::Status(s)
    }
}

impl From<protocol::Stats> for EnumResponse {
    fn from(s: Stats) -> Self {
        EnumResponse::Stats(s)
    }
}
