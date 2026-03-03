
mod cmd_cy;
mod cmd_z0;

pub mod command;

pub use command::*;
pub use cmd_cy::{VerifyCVVRequest, VerifyCVVResponse};
pub use cmd_z0::{GetKeyRequest, GetKeyResponse};


use anyhow::{Result, bail, Context};
use crate::message::header::MessageHeader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    VerifyCVVRequest(VerifyCVVRequest),
    VerifyCVVResponse(VerifyCVVResponse),
    GetKeyRequest(GetKeyRequest),
    GetKeyResponse(GetKeyResponse),
}

impl Message {
    pub fn parse(buffer: &[u8]) -> Result<Self> {
        let header = MessageHeader::parse(buffer)
            .context("Failed to parse message header")?;

        match &header.cmd {
            &CMD_VERIFYCVV_REQUEST => {
                Ok(Message::VerifyCVVRequest(VerifyCVVRequest::parse(buffer)?))
            }
            &CMD_VERIFYCVV_RESPONSE => {
                Ok(Message::VerifyCVVResponse(VerifyCVVResponse::parse(buffer)?))
            }
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
            Message::VerifyCVVRequest(req)  => req.to_bytes(),
            Message::VerifyCVVResponse(res) => res.to_bytes(),
            Message::GetKeyRequest(req)     => req.to_bytes(),
            Message::GetKeyResponse(resp)   => resp.to_bytes(),
        }
    }

    pub fn cmd(&self) -> String {
        match self {
            Message::VerifyCVVRequest(req)  => req.header.cmd_str(),
            Message::VerifyCVVResponse(res) => res.header.cmd_str(),
            Message::GetKeyRequest(req)     => req.header.cmd_str(),
            Message::GetKeyResponse(resp)   => resp.header.cmd_str(),
        }
    }
}