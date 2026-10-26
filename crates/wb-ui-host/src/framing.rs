//! Finite length-prefixed UI transport; accepted bus remains inside the worker.
use std::io::{self, Read, Write};
pub const MAX_FRAME: usize = 65536;
pub const DEFAULT_CAPACITY: usize = 128;
pub fn read_frame(mut input: impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    let mut count = 0;
    while count < 4 {
        match input.read(&mut header[count..])? {
            0 if count == 0 => return Ok(None),
            0 => return Err(io::ErrorKind::UnexpectedEof.into()),
            n => count += n,
        }
    }
    let size = u32::from_be_bytes(header) as usize;
    if size == 0 || size > MAX_FRAME {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut bytes = vec![0; size];
    input.read_exact(&mut bytes)?;
    Ok(Some(bytes))
}
pub fn write_frame(mut output: impl Write, bytes: &[u8]) -> io::Result<()> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(io::ErrorKind::InvalidData.into());
    }
    output.write_all(&(bytes.len() as u32).to_be_bytes())?;
    output.write_all(bytes)?;
    output.flush()
}
