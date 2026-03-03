use anyhow::{Result, bail, Context};

use crate::message::header::{MessageHeader, MSGHDR_FMT_SIZE, MSGHDR_LEN_SIZE};
use super::command::{CMD_VERIFYCVV_REQUEST, CMD_VERIFYCVV_RESPONSE, RESPONSE_SUCCESS};


// Fixed-size fields: cvka(16) + cvkb(16) + cvv(3) + expdate(4) + svcode(3) = 42
// Plus variable: pan digits + ';'
pub const VERIFYCVV_FIXED_FIELDS_SIZE: usize = 16 + 16 + 3 + 4 + 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyCVVRequest {
    pub header: MessageHeader,
    pub cvka: [u8; 16],
    pub cvkb: [u8; 16],
    pub cvv:  [u8; 3],
    pub pan: Vec<u8>,
    pub expdate: [u8; 4],
    pub svcode: [u8; 3]
}

impl VerifyCVVRequest {

    fn validate_cvka(cvka: &str) -> Result<[u8; 16]> {
        if cvka.is_empty() || cvka.len() > 16 {
            bail!("CVKA must be 1-16 bytes, got {}", cvka.len());
        }
        let mut buf = [0u8; 16];
        buf[..cvka.len()].copy_from_slice(cvka.as_bytes());
        Ok(buf)
    }

    fn validate_cvkb(cvkb: &str) -> Result<[u8; 16]> {
        if cvkb.is_empty() || cvkb.len() > 16 {
            bail!("CVKB must be 1-16 bytes, got {}", cvkb.len());
        }
        let mut buf = [0u8; 16];
        buf[..cvkb.len()].copy_from_slice(cvkb.as_bytes());
        Ok(buf)
    }

    fn validate_cvv(cvv: &str) -> Result<[u8; 3]> {
        if cvv.is_empty() || cvv.len() > 3 {
            bail!("CVV must be 1-3 digits, got {}", cvv.len());
        }
        if !cvv.chars().all(|c| c.is_ascii_digit()) {
            bail!("CVV must contain digits only, got: {}", cvv);
        }
        let mut buf = [0u8; 3];
        buf[..cvv.len()].copy_from_slice(cvv.as_bytes());
        Ok(buf)
    }

    fn validate_pan(pan: &str) -> Result<Vec<u8>> {
        if pan.is_empty() || pan.len() > 19 {
            bail!("PAN must be 1-19 digits, got {}", pan.len());
        }
        if !pan.chars().all(|c| c.is_ascii_digit()) {
            bail!("PAN must contain digits only, got: {}", pan);
        }
        Ok(pan.as_bytes().to_vec())
    }

    fn validate_expdate(expdate: &str) -> Result<[u8; 4]> {
        if expdate.len() != 4 {
            bail!("Expdate must be exactly 4 bytes (YYMM), got {}", expdate.len());
        }
        if !expdate.chars().all(|c| c.is_ascii_digit()) {
            bail!("Expdate must contain digits only, got: {}", expdate);
        }
        expdate.as_bytes().try_into()
            .context("Failed to convert expdate to [u8; 4]")
    }

    fn validate_svcode(svcode: &str) -> Result<[u8; 3]> {
        if svcode.len() != 3 {
            bail!("Service code must be exactly 3 bytes, got {}", svcode.len());
        }
        if !svcode.chars().all(|c| c.is_ascii_digit()) {
            bail!("Service code must contain digits only, got: {}", svcode);
        }
        svcode.as_bytes().try_into()
            .context("Failed to convert svcode to [u8; 3]")
    }

    pub fn new(
        hdr     : [u8; 4],
        cvka    : &str,
        cvkb    : &str,
        cvv     : &str,
        pan     : &str,
        expdate : &str,
        svcode  : &str,
    ) -> Result<Self> {
        let cvka    = Self::validate_cvka(cvka)?;
        let cvkb    = Self::validate_cvkb(cvkb)?;
        let cvv     = Self::validate_cvv(cvv)?;
        let pan     = Self::validate_pan(pan)?;
        let expdate = Self::validate_expdate(expdate)?;
        let svcode  = Self::validate_svcode(svcode)?;

        let payload_len = VERIFYCVV_FIXED_FIELDS_SIZE + pan.len() + 1;
        let header = MessageHeader::new(hdr, CMD_VERIFYCVV_REQUEST, payload_len);

        Ok(Self { header, cvka, cvkb, cvv, pan, expdate, svcode })
    }

    pub fn parse(buffer: &[u8]) -> Result<Self> {
        let header = MessageHeader::parse(buffer)
            .context("Failed to parse message header")?;

        if !header.is_cmd(&CMD_VERIFYCVV_REQUEST) {
            bail!("Invalid command: expected VerifyCVV, got {}", header.cmd_str());
        }

        let mut offset = MSGHDR_FMT_SIZE;

        let cvka = Self::validate_cvka(
            std::str::from_utf8(&buffer[offset..offset + 16]).context("Invalid UTF-8 in cvka")?
        )?;
        offset += 16;

        let cvkb = Self::validate_cvkb(
            std::str::from_utf8(&buffer[offset..offset + 16]).context("Invalid UTF-8 in cvkb")?
        )?;
        offset += 16;

        let cvv = Self::validate_cvv(
            std::str::from_utf8(&buffer[offset..offset + 3]).context("Invalid UTF-8 in cvv")?
        )?;
        offset += 3;

        let pan_start = offset;
        let pan_end = buffer[pan_start..].iter().position(|&b| b == b';')
            .map(|pos| pan_start + pos)
            .context("PAN delimiter ';' not found")?;
        let pan = Self::validate_pan(
            std::str::from_utf8(&buffer[pan_start..pan_end]).context("Invalid UTF-8 in pan")?
        )?;
        offset = pan_end + 1;

        let expdate = Self::validate_expdate(
            std::str::from_utf8(&buffer[offset..offset + 4]).context("Invalid UTF-8 in expdate")?
        )?;
        offset += 4;

        let svcode = Self::validate_svcode(
            std::str::from_utf8(&buffer[offset..offset + 3]).context("Invalid UTF-8 in svcode")?
        )?;

        Ok(Self { header, cvka, cvkb, cvv, pan, expdate, svcode })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let total = MSGHDR_LEN_SIZE + VERIFYCVV_FIXED_FIELDS_SIZE + self.pan.len() + 1;
        let mut result = Vec::with_capacity(total);
        result.extend_from_slice(&self.header.to_bytes());
        result.extend_from_slice(&self.cvka);
        result.extend_from_slice(&self.cvkb);
        result.extend_from_slice(&self.cvv);
        result.extend_from_slice(&self.pan); // raw digits
        result.push(b';');                   // terminator
        result.extend_from_slice(&self.expdate);
        result.extend_from_slice(&self.svcode);
        result
    }
}






#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyCVVResponse {
    pub header: MessageHeader,
    pub response_code: [u8; 2],
}

impl VerifyCVVResponse {
    pub fn success(hdr: [u8; 4]) -> Self {
        let data_length = 2;
        let header = MessageHeader::new(hdr, CMD_VERIFYCVV_RESPONSE, data_length);
        
        Self {
            header,
            response_code: RESPONSE_SUCCESS,
        }
    }

    pub fn error(hdr: [u8; 4], error_code: [u8; 2]) -> Self {
        let data_length = 2;
        let header = MessageHeader::new(hdr, CMD_VERIFYCVV_RESPONSE, data_length);
        
        Self {
            header,
            response_code: error_code
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

        if !header.is_cmd(&CMD_VERIFYCVV_RESPONSE) {
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
        
        Ok(Self {
            header,
            response_code
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(MSGHDR_LEN_SIZE + self.header.len as usize);

        result.extend_from_slice(&self.header.to_bytes());
        result.extend_from_slice(&self.response_code);

        result
    }
}