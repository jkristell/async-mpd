use itertools::Itertools;
use std::str::FromStr;

use async_net::TcpStream;
use futures_lite::{
    io::BufReader,
    {AsyncBufReadExt, AsyncWriteExt},
};

use crate::{
    client::{resp::mixed_stream, respmap::RespMap, Command, CommandResponse},
    resp, Error,
};

pub async fn send_command(line: &str, reader: &mut BufReader<TcpStream>) -> std::io::Result<()> {
    // Get the underlying TcpStrem and write command to the socket
    reader.get_mut().write_all(line.as_bytes()).await
}

pub async fn handle_resp(
    cmd: &Command<'_>,
    reader: &mut BufReader<TcpStream>,
) -> Result<CommandResponse, crate::Error> {
    match cmd {
        Command::Stats => {
            let r = read_resp(reader).await?;
            let map = RespMap::from_string(r);
            Ok(CommandResponse::Stats(map.into()))
        }

        Command::Status => {
            let r = read_resp(reader).await?;
            let map = RespMap::from_string(r);
            Ok(CommandResponse::Status(map.into()))
        }

        Command::Update(_) | Command::Rescan(_) => {
            let r = read_resp_line(reader).await?;

            let db_version = match r.split(": ").next_tuple() {
                Some(("updating_db", dbv)) => dbv.parse().map_err(|_| Error::ValueError {
                    msg: "".to_string(),
                }),
                _ => {
                    return Err(Error::ValueError {
                        msg: "updating_db".to_string(),
                    })
                }
            }?;

            Ok(CommandResponse::DbVersion(db_version))
        }

        Command::Idle => {
            let resp = read_resp(reader).await?;
            let mut lines = resp.lines();

            let line = lines.next().unwrap_or_default();
            for unexpected_line in lines {
                log::warn!("More than one line in idle response: {}", unexpected_line);
            }

            if let Some((k, v)) = line.splitn(2, ": ").next_tuple() {
                if k != "changed" {
                    log::warn!("k not changed");
                    return Err(Error::CommandError {
                        msg: "".to_string(),
                    });
                }

                let subsystem = crate::Subsystem::from_str(v)?;
                return Ok(CommandResponse::Subsystem(subsystem));
            }
            Err(Error::CommandError {
                msg: "".to_string(),
            })
        }

        Command::SetVol(_)
        | Command::PlayId(_)
        | Command::Consume(_)
        | Command::NoIdle
        | Command::Repeat(_)
        | Command::Random(_)
        | Command::PlayPaus(_)
        | Command::Next
        | Command::Prev
        | Command::Stop
        | Command::QueueAdd(_)
        | Command::QueueClear => {
            read_ok_resp(reader).await?;
            Ok(CommandResponse::Ok)
        }

        Command::PlaylistInfo | Command::Search(_) => {
            Ok(CommandResponse::Tracks(resp::tracks(reader).await?))
        }

        Command::Listall(_) => {
            let r = read_resp(reader)
                .await?
                .lines()
                .filter_map(|line| {
                    if line.starts_with("file: ") {
                        Some(line[6..].to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(CommandResponse::Paths(r))
        }

        Command::ListallInfo(_) => {
            let resp = mixed_stream(reader).await?;
            Ok(CommandResponse::Mixed(resp))
        }
    }
}

/// Expect one line response
pub(crate) async fn read_resp_line(reader: &mut BufReader<TcpStream>) -> Result<String, Error> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    Ok(line.trim().to_string())
}

/// Read and expect OK response line
async fn read_ok_resp(reader: &mut BufReader<TcpStream>) -> Result<(), Error> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    if &line != "OK\n" {
        return Err(Error::ResponseError {
            reply: line.to_string(),
            errmsg: "Expected OK".to_string(),
        });
    }

    Ok(())
}

/// Read all response lines
async fn read_resp(reader: &mut BufReader<TcpStream>) -> Result<String, Error> {
    let mut v = Vec::new();

    loop {
        let mut line = String::new();

        if reader.read_line(&mut line).await? == 0 {
            return Err(Error::Disconnected);
        }

        let line = line.trim();

        if line == "OK" {
            break;
        }

        if line.starts_with("ACK ") {
            log::trace!("Cmd error: {}", line);
            return Err(Error::CommandError { msg: line.into() });
        }

        v.push(line.to_string())
    }

    Ok(v.join("\n"))
}
