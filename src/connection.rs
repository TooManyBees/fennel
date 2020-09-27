use generational_arena::Index;
use std::io::{prelude::*, ErrorKind, Result as IoResult, Write};
use std::net::{SocketAddr, TcpStream};

use crate::Player;
use serde::export::fmt::Arguments;

pub struct ConnectionBuilder {
    pub stream: TcpStream,
    pub addr: SocketAddr,
}

impl ConnectionBuilder {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> ConnectionBuilder {
        ConnectionBuilder { stream, addr }
    }

    pub fn logged_in(self, player: Player, char_idx: Index) -> Connection {
        Connection {
            stream: self.stream,
            addr: self.addr,
            player,
            character: char_idx,
            in_buffer: [0; 256],
            input: None,
            output: vec![],
        }
    }
}

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    player: Player,
    pub character: Index,
    in_buffer: [u8; 256],
    pub input: Option<String>, // TODO: do this better
    output: Vec<u8>,
}

impl Connection {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_name(&self) -> &str {
        self.player.name()
    }

    pub fn write_flush(&mut self) -> IoResult<()> {
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

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.stream.flush()
    }

    fn write_fmt(&mut self, fmt: Arguments<'_>) -> IoResult<()> {
        self.stream.write_fmt(fmt)
    }
}
