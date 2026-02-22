
mod z01;

pub mod command;

pub use command::*;
pub use z01::{GetKeyRequest, GetKeyResponse};

use anyhow::{Result, bail, Context};
use crate::message::header::MessageHeader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    GetKeyRequest(GetKeyRequest),
    GetKeyResponse(GetKeyResponse),
}

impl Message {
    pub fn parse(buffer: &[u8]) -> Result<Self> {
        let header = MessageHeader::parse(buffer)
            .context("Failed to parse message header")?;

        match &header.cmd {
            &CMD_GETKEY_REQUEST => {
                Ok(Message::GetKeyRequest(GetKeyRequest::parse(buffer)?))
            }
            &CMD_GETKEY_RESPONSE => {
                Ok(Message::GetKeyResponse(GetKeyResponse::parse(buffer)?))
            }
            _ => {
                bail!("Unknown command: {}", header.cmd_str());
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::GetKeyRequest(req)   => req.to_bytes(),
            Message::GetKeyResponse(resp) => resp.to_bytes(),
        }
    }

    pub fn cmd(&self) -> String {
        match self {
            Message::GetKeyRequest(req) => req.header.cmd_str(),
            Message::GetKeyResponse(resp) => resp.header.cmd_str(),
        }
    }
}