use anyhow::Result;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;


use crate::io_result::IoResult;

pub async fn read<S>(
    stream: &mut S,
    buffer: &mut [u8],
    expected_length: usize,
    timeout: Option<Duration>,
    shutdown_token: &CancellationToken,
    context: &str
) -> Result<IoResult>
where 
    S: AsyncRead + Unpin
{
    if buffer.len() < expected_length {
        anyhow::bail!("Buffer too small: need {}, have {}", expected_length, buffer.len());
    }

    let mut total_bytes = 0;
    let mut remaining   = expected_length;
    let start           = std::time::Instant::now();
    
    while remaining > 0 {
        let remaining_timeout = match timeout {
            Some(t) => {
                let elapsed = start.elapsed();
                if elapsed >= t {
                    log::warn!(
                        "Timeout reading {} after {}s (got {}/{} bytes)", context, t.as_secs(), total_bytes, expected_length
                    );
                    return Ok(IoResult::Timeout);
                }
                Some(t - elapsed)
            }
            None => None,
        };


        match remaining_timeout {
            Some(timeout_duration) => {
                tokio::select! {
                    result = stream.read(&mut buffer[total_bytes..expected_length]) => {
                        match result {
                            Ok(0) => {
                                log::debug!("Connection closed during {} (got {}/{} bytes)", context, total_bytes, expected_length);
                                return Ok(IoResult::Closed);
                            }
                            Ok(n) => {
                                log::trace!("Read {} bytes of {} ({}/{})", n, context, total_bytes + n, expected_length);
                                total_bytes += n;
                                remaining -= n;
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to read {} at byte {}/{}: {}",context, total_bytes, expected_length, e));
                            }
                        }
                    }
                    
                    _ = shutdown_token.cancelled() => {
                        anyhow::bail!("Shutdown requested during {} (got {}/{} bytes)", context, total_bytes, expected_length);
                    }
                    
                    _ = tokio::time::sleep(timeout_duration) => {
                        log::warn!("Timeout reading {} (got {}/{} bytes)", context, total_bytes, expected_length);
                        return Ok(IoResult::Timeout);
                    }
                }
            }
            None => {
                tokio::select! {
                    result = stream.read(&mut buffer[total_bytes..expected_length]) => {
                        match result {
                            Ok(0) => {
                                log::debug!("Connection closed during {} (got {}/{} bytes)", context, total_bytes, expected_length);
                                return Ok(IoResult::Closed);
                            }
                            Ok(n) => {
                                log::trace!("Read {} bytes of {} ({}/{})", n, context, total_bytes + n, expected_length);
                                total_bytes += n;
                                remaining -= n;
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to read {} at byte {}/{}: {}", context, total_bytes, expected_length, e));
                            }
                        }
                    }
                    
                    _ = shutdown_token.cancelled() => {
                        anyhow::bail!(
                            "Shutdown requested during {} (got {}/{} bytes)",
                            context, total_bytes, expected_length
                        );
                    }
                }
            }
        }
    }
   
    log::debug!("Successfully read {} ({} bytes)", context, expected_length);
    Ok(IoResult::Success(total_bytes))
}

pub async fn write<S>(
    stream: &mut S,
    buffer: &[u8],
    timeout: Option<Duration>,
    shutdown_token: &CancellationToken,
    context: &str,
) -> Result<IoResult>
where
    S: AsyncWrite + Unpin,
{
    let mut total_bytes = 0;
    let mut remaining   = buffer.len();
    let start           = std::time::Instant::now();

    while remaining > 0 {
        let remaining_timeout = match timeout {
            Some(t) => {
                let elapsed = start.elapsed();
                if elapsed >= t {
                    log::warn!("Timeout writing {} after {}s (sent {}/{} bytes)", context, t.as_secs(), total_bytes, buffer.len());
                    return Ok(IoResult::Timeout);
                }
                Some(t - elapsed)
            }
            None => None,
        };
        
        match remaining_timeout {
            Some(timeout_duration) => {
                tokio::select! {
                    result = stream.write(&buffer[total_bytes..]) => {
                        match result {
                            Ok(0) => {
                                log::warn!("Connection closed during {} (sent {}/{} bytes)", context, total_bytes, buffer.len());
                                return Ok(IoResult::Closed);
                            }
                            Ok(n) => {
                                log::trace!("Wrote {} bytes of {} ({}/{})", n, context, total_bytes + n, buffer.len());
                                total_bytes += n;
                                remaining -= n;
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to write {} at byte {}/{}: {}", context, total_bytes, buffer.len(), e));
                            }
                        }
                    }
                    
                    _ = shutdown_token.cancelled() => {
                        anyhow::bail!("Shutdown requested during {} (sent {}/{} bytes)", context, total_bytes, buffer.len());
                    }
                    
                    _ = tokio::time::sleep(timeout_duration) => {
                        log::warn!("Timeout writing {} (sent {}/{} bytes)", context, total_bytes, buffer.len());
                        return Ok(IoResult::Timeout);
                    }
                }
            }
            None => {
                tokio::select! {
                    result = stream.write(&buffer[total_bytes..]) => {
                        match result {
                            Ok(0) => {
                                log::warn!("Connection closed during {} (sent {}/{} bytes)", context, total_bytes, buffer.len());
                                return Ok(IoResult::Closed);
                            }
                            Ok(n) => {
                                log::trace!("Wrote {} bytes of {} ({}/{})", n, context, total_bytes + n, buffer.len());
                                total_bytes += n;
                                remaining -= n;
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to write {} at byte {}/{}: {}", context, total_bytes, buffer.len(), e));
                            }
                        }
                    }
                    
                    _ = shutdown_token.cancelled() => {
                        anyhow::bail!("Shutdown requested during {} (sent {}/{} bytes)", context, total_bytes, buffer.len());
                    }
                }
            }
        }
    }
    
    let remaining_timeout = match timeout {
        Some(t) => {
            let elapsed = start.elapsed();
            if elapsed >= t {
                log::warn!("Timeout before flush ({}s elapsed)", elapsed.as_secs());
                return Ok(IoResult::Timeout);
            }
            Some(t - elapsed)
        }
        None => None,
    };

    match remaining_timeout {
        Some(timeout_duration) => {
            tokio::select! {
                result = stream.flush() => {
                    result.map_err(|e| anyhow::anyhow!("Failed to flush {}: {}", context, e))?;
                }
                _ = shutdown_token.cancelled() => {
                    anyhow::bail!("Shutdown requested during {} flush", context);
                }
                _ = tokio::time::sleep(timeout_duration) => {
                    log::warn!("Timeout flushing {}", context);
                    return Ok(IoResult::Timeout);
                }
            }
        }
        None => {
            tokio::select! {
                result = stream.flush() => {
                    result.map_err(|e| anyhow::anyhow!("Failed to flush {}: {}", context, e))?;
                }
                _ = shutdown_token.cancelled() => {
                    anyhow::bail!("Shutdown requested during {} flush", context);
                }
            }
        }
    } 

    log::debug!("Successfully wrote {} ({} bytes in {:?})", context, total_bytes, start.elapsed());    
    Ok(IoResult::Success(total_bytes))
}