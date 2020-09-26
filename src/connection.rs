use generational_arena::Index;
use std::fmt::Display;
use std::io::{prelude::*, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    pub character: Option<Index>,
    in_buffer: [u8; 256],
    pub input: Option<String>, // TODO: do this better
    output: Vec<u8>,
}

impl Connection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Connection {
        Connection {
            stream,
            addr,
            character: None,
            in_buffer: [0; 256],
            input: None,
            output: Vec::with_capacity(500),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn write(&mut self, message: &dyn Display) -> std::io::Result<()> {
        write!(&mut self.output, "{}\r\n", message)
    }

    pub fn write_flush(&mut self) -> std::io::Result<()> {
        if !self.output.is_empty() {
            self.stream.write_all(&self.output)?;
            self.output.clear(); // TODO: find a way to shrink capacity down to 500 if poss?
        }
        Ok(())
    }

    pub fn read(&mut self) -> std::io::Result<String> {
        // FIXME: prevent input overflows; max length should be 256
        let n = self.stream.read(&mut self.in_buffer)?;
        let s = String::from_utf8(self.in_buffer[..n].to_vec())
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8"))?;
        Ok(s)
    }
}
