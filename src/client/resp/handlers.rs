use async_net::TcpStream;
use async_trait::async_trait;

use futures_lite::io::BufReader;
use futures_lite::{AsyncBufReadExt, StreamExt};

use std::marker::PhantomData;
use std::str::FromStr;

use crate::resp::EnumResponse;
use crate::{
    client::resp::{
        read_resp_line,
        respmap::RespMap,
        respmap_handlers::{mixed_stream, tracks, ListallinfoResponse},
    },
    Error, Track,
};

#[async_trait]
/// Response Handler for Cmd
pub trait ResponseHandler: Sized {
    /// The type of response
    type Response: Into<EnumResponse>;

    async fn handle(reader: &mut BufReader<TcpStream>) -> Result<Self::Response, crate::Error>;
}

pub struct Tracks;

#[async_trait]
impl ResponseHandler for Tracks {
    type Response = Vec<Track>;

    async fn handle(reader: &mut BufReader<TcpStream>) -> Result<Self::Response, Error> {
        tracks(reader).await.map_err(Into::into)
    }
}

pub struct MixedResponseResponse;

#[async_trait]
impl ResponseHandler for MixedResponseResponse {
    type Response = ListallinfoResponse;

    async fn handle(reader: &mut BufReader<TcpStream>) -> Result<Self::Response, Error> {
        mixed_stream(reader).await.map_err(Into::into)
    }
}

pub struct RespMapResponse<T> {
    _0: PhantomData<T>,
}

#[async_trait]
impl<T: From<RespMap> + Into<EnumResponse>> ResponseHandler for RespMapResponse<T> {
    type Response = T;

    async fn handle(reader: &mut BufReader<TcpStream>) -> Result<Self::Response, Error> {
        let mut map = RespMap::new();
        let mut lines = reader.lines();

        while let Some(line) = lines.next().await {
            let line = line?;
            log::debug!("line: '{}'", line);

            if &line == "OK" {
                break;
            }

            if line.starts_with("ACK ") {
                return Err(crate::Error::ServerError { msg: line });
            }

            if let Some((k, v)) = line.split_once(": ") {
                map.insert(k, v);
            }
        }

        Ok(map.into())
    }
}

pub struct SingleLineResp<T> {
    _0: PhantomData<T>,
}

#[async_trait]
impl<E: Into<crate::Error>, T: FromStr<Err = E> + Into<EnumResponse>> ResponseHandler
    for SingleLineResp<T>
{
    type Response = T;

    async fn handle(reader: &mut BufReader<TcpStream>) -> Result<Self::Response, Error> {
        let line = read_resp_line(reader).await?;
        let value = line.split(": ").skip(1).next().unwrap_or_default();
        T::from_str(&value).map_err(Into::into)
    }
}

pub struct OkResponse;

#[async_trait]
impl ResponseHandler for OkResponse {
    type Response = ();

    async fn handle(reader: &mut BufReader<TcpStream>) -> Result<Self::Response, crate::Error> {
        let mut lines = reader.lines();

        if let Some(line) = lines.next().await {
            let line = line?;

            if &line == "OK" {
                Ok(())
            } else {
                Err(crate::Error::ServerError { msg: line })
            }
        } else {
            Ok(())
        }
    }
}
