
use anyhow::{Result, bail};


pub const MSGHDR_LEN_SIZE: usize = 2;
pub const MSGHDR_HDR_SIZE: usize = 4;
pub const MSGHDR_CMD_SIZE: usize = 2;
pub const MSGHDR_FMT_SIZE: usize = MSGHDR_LEN_SIZE + MSGHDR_HDR_SIZE + MSGHDR_CMD_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub len: u16,
    pub hdr: [u8; 4],
    pub cmd: [u8; 2],
}

impl MessageHeader {
    pub fn new(hdr: [u8; 4], cmd: [u8; 2], data_length: usize) -> Self {
        let len = (MSGHDR_HDR_SIZE + MSGHDR_CMD_SIZE + data_length) as u16;
        Self { len, hdr, cmd }
    }

    pub fn cmd_str(&self) -> String {
        String::from_utf8_lossy(&self.cmd).to_string()
    }

    pub fn is_cmd(&self, cmd: &[u8; 2]) -> bool {
        &self.cmd == cmd
    }

    pub fn data_length(&self) -> usize {
        self.len as usize - MSGHDR_HDR_SIZE - MSGHDR_CMD_SIZE
    }

    pub fn parse(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < MSGHDR_FMT_SIZE {
            bail!("Buffer too small for header: need {}, got {}", MSGHDR_FMT_SIZE, buffer.len());
        }
        
        let len = u16::from_be_bytes([buffer[0], buffer[1]]);
        
        let mut hdr = [0u8; 4];
        hdr.copy_from_slice(&buffer[2..6]);
        
        let mut cmd = [0u8; 2];
        cmd.copy_from_slice(&buffer[6..8]);
        
        Ok(Self { len, hdr, cmd })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(MSGHDR_FMT_SIZE);

        result.extend_from_slice(&self.len.to_be_bytes());
        result.extend_from_slice(&self.hdr);
        result.extend_from_slice(&self.cmd);
        result
    }
}
