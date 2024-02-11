use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

pub trait InsertDylibFileExt {
    /// Read exactly `buf.len()` bytes and then rewind the file cursor `buf.len()` bytes
    fn fpeek(&mut self, buf: &mut [u8]) -> io::Result<()>;
    /// Overwrite `len` zeros to file starting from `offset` (from beginning)
    fn fbzero(&mut self, offset: u64, len: u64) -> io::Result<()>;
    /// memmove alike, but on file
    fn fmemmove(&mut self, dst: u64, src: u64, len: u64) -> io::Result<()>;
    /// Get current file cursor position
    fn ftello(&mut self) -> u64;
}

impl InsertDylibFileExt for File {
    fn fpeek(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.read_exact(buf)?;
        self.seek(SeekFrom::Current(0i64 - (buf.len() as i64)))?;
        Ok(())
    }

    fn fbzero(&mut self, offset: u64, len: u64) -> io::Result<()> {
        static ZEROS: [u8; 512] = [0u8; 512];
        self.seek(SeekFrom::Start(offset))?;
        let mut remain = len as usize;
        while remain > 0 {
            if remain >= 512 {
                remain -= self.write(&ZEROS)?;
            } else {
                let zeros: Vec<u8> = vec![0; remain];
                remain -= self.write(&zeros)?;
            }
        }
        Ok(())
    }

    fn fmemmove(&mut self, dst: u64, src: u64, len: u64) -> io::Result<()> {
        const BUFSIZE: u64 = 512;
        let mut buf = [0u8; BUFSIZE as usize];
        let mut remain = len;
        let mut src = src;
        let mut dst = dst;
        while remain > 0 {
            if remain < 512 {
                self.seek(SeekFrom::Start(src))?;
                let mut small_buf: Vec<u8> = vec![0; remain as usize];
                self.read_exact(&mut small_buf)?;
                self.seek(SeekFrom::Start(dst))?;
                self.write_all(&small_buf)?;

                remain -= remain;
                src += remain;
                dst += remain;
            } else {
                self.seek(SeekFrom::Start(src))?;
                self.read_exact(&mut buf)?;
                self.seek(SeekFrom::Start(dst))?;
                self.write_all(&buf)?;

                remain -= BUFSIZE;
                src += BUFSIZE;
                dst += BUFSIZE;
            }
        }

        Ok(())
    }

    fn ftello(&mut self) -> u64 {
        self.stream_position().unwrap()
    }
}
