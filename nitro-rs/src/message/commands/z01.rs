use anyhow::{Result, bail, Context};

use crate::message::header::{MessageHeader, MSGHDR_FMT_SIZE, MSGHDR_LEN_SIZE};
use super::command::{CMD_GETKEY_REQUEST, CMD_GETKEY_RESPONSE, RESPONSE_SUCCESS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetKeyRequest {
    pub header: MessageHeader,
    pub key_id: Vec<u8>,
}

impl GetKeyRequest {
    pub fn new(hdr: [u8; 4], key_id: Vec<u8>) -> Self {
        let header = MessageHeader::new(hdr, CMD_GETKEY_REQUEST, key_id.len());
        Self { header, key_id }
    }

    pub fn key_id_str(&self) -> String {
        String::from_utf8_lossy(&self.key_id).to_string()
    }

    pub fn parse(buffer: &[u8]) -> Result<Self> {
        let header = MessageHeader::parse(buffer)
            .context("Failed to parse message header")?;

        if !header.is_cmd(&CMD_GETKEY_REQUEST) {
            bail!("Invalid command for KeyRequest: expected Z0, got {}", header.cmd_str());
        }

        let expected_total = MSGHDR_LEN_SIZE + header.len as usize;
        if buffer.len() < expected_total {
            bail!("Buffer too small: need {}, got {}", expected_total, buffer.len());
        }

        let data_length = header.data_length();
        let key_id = buffer[MSGHDR_FMT_SIZE..MSGHDR_FMT_SIZE + data_length].to_vec();
        
        Ok(Self { header, key_id })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(MSGHDR_LEN_SIZE + self.header.len as usize);
        result.extend_from_slice(&self.header.to_bytes());
        result.extend_from_slice(&self.key_id);
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetKeyResponse {
    pub header: MessageHeader,
    pub response_code: [u8; 2],
    pub encrypted_key: Option<Vec<u8>>,
}

impl GetKeyResponse {
    pub fn success(hdr: [u8; 4], encrypted_key: Vec<u8>) -> Self {
        let data_length = 2 + encrypted_key.len();
        let header = MessageHeader::new(hdr, CMD_GETKEY_RESPONSE, data_length);
        
        Self {
            header,
            response_code: RESPONSE_SUCCESS,
            encrypted_key: Some(encrypted_key),
        }
    }

    pub fn error(hdr: [u8; 4], error_code: [u8; 2]) -> Self {
        let data_length = 2;
        let header = MessageHeader::new(hdr, CMD_GETKEY_RESPONSE, data_length);
        
        Self {
            header,
            response_code: error_code,
            encrypted_key: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.response_code == RESPONSE_SUCCESS
    }

    pub fn response_code_str(&self) -> String {
        String::from_utf8_lossy(&self.response_code).to_string()
    }

    pub fn parse(buffer: &[u8]) -> Result<Self> {

        let header = MessageHeader::parse(buffer)
            .context("Failed to parse message header")?;

        if !header.is_cmd(&CMD_GETKEY_RESPONSE) {
            bail!("Invalid command for KeyResponse: expected Z1, got {}", header.cmd_str());
        }

        let expected_total = MSGHDR_LEN_SIZE + header.len as usize;
        if buffer.len() < expected_total {
            bail!("Buffer too small: need {}, got {}", expected_total, buffer.len());
        }
        
        let data_length = header.data_length();
        if data_length < 2 {
            bail!("Data too small for response code: need at least 2 bytes, got {}", data_length
            );
        }

        let mut response_code = [0u8; 2];
        response_code.copy_from_slice(&buffer[MSGHDR_FMT_SIZE..MSGHDR_FMT_SIZE + 2]);

        let encrypted_key = if data_length > 2 {
            let key_start = MSGHDR_FMT_SIZE + 2;
            let key_end = key_start + (data_length - 2);
            Some(buffer[key_start..key_end].to_vec())
        } else {
            None
        };
        
        Ok(Self {
            header,
            response_code,
            encrypted_key,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(MSGHDR_LEN_SIZE + self.header.len as usize);
        result.extend_from_slice(&self.header.to_bytes());
        result.extend_from_slice(&self.response_code);
        
        if let Some(ref key) = self.encrypted_key {
            result.extend_from_slice(key);
        }
        
        result
    }
}