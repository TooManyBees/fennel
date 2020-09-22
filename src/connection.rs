use generational_arena::Index;
use std::io::{prelude::*, ErrorKind};
use std::net::{SocketAddr, TcpStream};

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    pub character: Option<Index>,
    in_buffer: [u8; 256],
    input: Option<String>, // TODO: do this better
    output: Option<String>,
}

impl Connection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Connection {
        Connection {
            stream,
            addr,
            character: None,
            in_buffer: [0; 256],
            input: None,
            output: None,
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn write(&mut self, message: &str) -> std::io::Result<()> {
        self.stream.write_all(message.as_bytes())
    }

    pub fn read(&mut self) -> std::io::Result<()> {
        // FIXME: prevent input overflows; max length should be 256
        let n = self.stream.read(&mut self.in_buffer)?;
        let s = String::from_utf8(self.in_buffer[..n].to_vec())
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8"))?;
        self.input = Some(s);
        Ok(())
    }
}
