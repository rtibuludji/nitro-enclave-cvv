use anyhow::Result;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;


use crate::io_result::IoResult;
use crate::io_utils;

pub async fn read_message<S>(
    stream: &mut S,
    timeout: Option<Duration>,
    shutdown_token: &CancellationToken
) -> Result<Vec<u8>>
where 
    S: AsyncRead + Unpin
{
    const HEADER_LENGTH: usize = 2;
    let mut header = [0u8; HEADER_LENGTH];
    
    let mut context: &str = "read_header";
    match io_utils::read(stream, &mut header, HEADER_LENGTH, timeout, shutdown_token, context).await? {
        IoResult::Success(_) => {
        }
        IoResult::Closed => {
            log::debug!("Connection closed while reading header in {}", context);
            return Ok(Vec::new());
        }
        IoResult::Timeout => {
            log::warn!("Timeout reading header in {}", context);
            return Ok(Vec::new());
        }
    }
    
    let length = u16::from_be_bytes([header[0], header[1]]) as usize;
    
    log::debug!("Header received in {}: length={} bytes", context, length);
    
    // Validate length
    const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;  // 10 MB
    if length > MAX_MESSAGE_SIZE {
        anyhow::bail!("Message too large in {}: {} bytes (max {})", context, length, MAX_MESSAGE_SIZE);
    }

    context = "read_data";

    let mut data = vec![0u8; length];    
    match io_utils::read(stream, &mut data, length, timeout, shutdown_token, context).await? {
        IoResult::Success(_) => {
        }
        IoResult::Closed => {
            log::warn!("Connection closed while reading body in {}", context);
            return Ok(Vec::new());
        }
        IoResult::Timeout => {
            log::warn!("Timeout reading body in {}", context);
            return Ok(Vec::new());
        }
    }
    
    log::debug!("Body received in {}: {} bytes", context, data.len());
    
    // Return header + data
    let mut result = Vec::with_capacity(2 + length);
    result.extend_from_slice(&header);  // 2 bytes header
    result.extend_from_slice(&data);    // N bytes data
    
    Ok(result)
}

pub async fn write_message<S>(
    stream: &mut S,
    buffer: &[u8],
    timeout: Option<Duration>,
    shutdown_token: &CancellationToken
) -> Result<usize>
where 
    S: AsyncWrite + Unpin
{
    let context: &str = "read_header";
    match io_utils::write(stream, buffer, timeout, shutdown_token, context).await? {
        IoResult::Success(written) => {
            return Ok(written)
        }
        IoResult::Closed => {
            log::warn!("Connection closed while writing data {}", context);
            return Ok(0);
        }
        IoResult::Timeout => {
            log::warn!("Timeout reading body in {}", context);
            return Ok(0);
        }
    }    
}