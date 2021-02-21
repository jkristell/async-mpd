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

/// A Enum-wrapped response
pub enum WrappedResponse {
    Ok,
    ListAllInfo(ListallinfoResponse),
    Tracks(Vec<Track>),
    Listall(ListallResponse),
    Subsystem(Subsystem),
    DatabaseVersion(DatabaseVersion),
    Status(Status),
    Stats(Stats),
}

impl From<()> for WrappedResponse {
    fn from(_: ()) -> Self {
        WrappedResponse::Ok
    }
}

impl From<ListallinfoResponse> for WrappedResponse {
    fn from(l: ListallinfoResponse) -> Self {
        WrappedResponse::ListAllInfo(l)
    }
}

impl From<Vec<protocol::Track>> for WrappedResponse {
    fn from(t: Vec<Track>) -> Self {
        WrappedResponse::Tracks(t)
    }
}

impl From<ListallResponse> for WrappedResponse {
    fn from(l: ListallResponse) -> Self {
        WrappedResponse::Listall(l)
    }
}

impl From<protocol::Subsystem> for WrappedResponse {
    fn from(s: Subsystem) -> Self {
        WrappedResponse::Subsystem(s)
    }
}

impl From<protocol::DatabaseVersion> for WrappedResponse {
    fn from(d: DatabaseVersion) -> Self {
        WrappedResponse::DatabaseVersion(d)
    }
}

impl From<protocol::Status> for WrappedResponse {
    fn from(s: Status) -> Self {
        WrappedResponse::Status(s)
    }
}

impl From<protocol::Stats> for WrappedResponse {
    fn from(s: Stats) -> Self {
        WrappedResponse::Stats(s)
    }
}
