mod byte;
mod event;
mod negotiation;
mod option;

use byte::*;
pub use event::TelnetEvent;
use event::TelnetEventQueue;
pub use option::TelnetOption;
pub use negotiation::NegotiationAction;
use std::net::TcpStream;
use std::io::{self, ErrorKind, Read, Write};
use smol::{Async};
use smol::prelude::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
enum ProcessState {
    NormalData,
    IAC,
    SB,
    SBData(TelnetOption, usize),    // (option, start location of option data)
    SBDataIAC(TelnetOption, usize), // (option, start location of option data)
    Will, Wont,
    Do, Dont
}


pub struct Telnet<S> {
    stream: S,
    event_queue: TelnetEventQueue,

    // Buffer
    buffer: Box<[u8]>,
    buffered_size: usize,
    process_buffer: Box<[u8]>,
    process_buffered_size: usize
}

impl<S> Telnet<S> {
    pub fn from_stream(stream: S, buf_size: usize) -> Telnet<S> {
        let actual_size = if buf_size == 0 { 1 } else { buf_size };

        Telnet {
            stream,
            event_queue: TelnetEventQueue::new(),
            buffer: vec![0; actual_size].into_boxed_slice(),
            buffered_size: 0,
            process_buffer: vec![0; actual_size].into_boxed_slice(),
            process_buffered_size: 0,
        }
    }

    fn process(&mut self) {
        let mut current = 0;
        let mut state = ProcessState::NormalData;
        let mut data_start = 0;

        while current < self.buffered_size {
            // Gather a byte
            let byte = self.buffer[current];

            // Process the byte
            match state {
                // Normal Data
                ProcessState::NormalData => {
                    if byte == BYTE_IAC {
                        // The following bytes will be commands

                        // Update the state
                        state = ProcessState::IAC;

                        // Send the data before this byte
                        if current > data_start {
                            let data_end = current;
                            let data = self.copy_buffered_data(data_start, data_end);
                            self.event_queue.push_event(TelnetEvent::Data(data));

                            // Update the state
                            data_start = current;
                        }
                    } else if current == self.buffered_size - 1 {
                        // It reaches the end of the buffer
                        let data_end = self.buffered_size;
                        let data = self.copy_buffered_data(data_start, data_end);
                        self.event_queue.push_event(TelnetEvent::Data(data));
                    }
                },

                // Telnet Commands
                ProcessState::IAC => {
                    match byte {
                        // Negotiation Commands
                        BYTE_WILL => state = ProcessState::Will,
                        BYTE_WONT => state = ProcessState::Wont,
                        BYTE_DO => state = ProcessState::Do,
                        BYTE_DONT => state = ProcessState::Dont,
                        // Subnegotiation
                        BYTE_SB => state = ProcessState::SB,
                        // Escaping
                        // TODO: Write a test case for this
                        BYTE_IAC => {
                            // Copy the data to the process buffer
                            self.append_data_to_proc_buffer(data_start, current - 1);

                            // Add escaped IAC
                            self.process_buffer[self.process_buffered_size] = BYTE_IAC;
                            self.process_buffered_size += 1;

                            // Update the state
                            state = ProcessState::NormalData;
                            data_start = current + 1;
                        },
                        // Unknown IAC commands
                        _ => {
                            state = ProcessState::NormalData;
                            data_start = current + 1;
                            self.event_queue.push_event(TelnetEvent::UnknownIAC(byte));
                        }
                    }
                },

                // Negotiation
                ProcessState::Will | ProcessState::Wont |
                ProcessState::Do | ProcessState::Dont => {

                    let opt = TelnetOption::parse(byte);

                    match state {
                        ProcessState::Will => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Will, opt));
                        },
                        ProcessState::Wont => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Wont, opt));
                        },
                        ProcessState::Do => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Do, opt));
                        },
                        ProcessState::Dont => {
                            self.event_queue.push_event(
                                TelnetEvent::Negotiation(NegotiationAction::Dont, opt));
                        },
                        _ => {} // Do nothing
                    }

                    state = ProcessState::NormalData;
                    data_start = current + 1;
                },

                // Start subnegotiation
                ProcessState::SB => {
                    let opt = TelnetOption::parse(byte);
                    state = ProcessState::SBData(opt, current + 1);
                },

                // Subnegotiation's data
                ProcessState::SBData(opt, data_start) => {
                    if byte == BYTE_IAC {
                        state = ProcessState::SBDataIAC(opt, data_start);
                    }

                    // XXX: We may need to consider the case that a SB Data
                    // sequence may exceed this buffer
                },

                // IAC inside Subnegotiation's data
                ProcessState::SBDataIAC(opt, sb_data_start) => {
                    match byte {
                        // The end of subnegotiation
                        BYTE_SE => {
                            // Update state
                            state = ProcessState::NormalData;
                            data_start = current + 1;

                            // Return the option
                            let sb_data_end = current - 1;
                            let data = self.copy_buffered_data(sb_data_start, sb_data_end);
                            self.event_queue.push_event(TelnetEvent::Subnegotiation(opt, data));
                        },
                        // Escaping
                        // TODO: Write a test case for this
                        BYTE_IAC => {
                            // Copy the data to the process buffer
                            self.append_data_to_proc_buffer(sb_data_start, current - 1);

                            // Add escaped IAC
                            self.process_buffer[self.process_buffered_size] = BYTE_IAC;
                            self.process_buffered_size += 1;

                            // Update the state
                            state = ProcessState::SBData(opt, current + 1);
                        },
                        // TODO: Write a test case for this
                        b => {
                            self.event_queue.push_event(TelnetEvent::Error(
                                format!("Unexpected byte after IAC inside SB: {}", b)));

                            // Copy the data to the process buffer
                            self.append_data_to_proc_buffer(sb_data_start, current - 1);
                            // Update the state
                            state = ProcessState::SBData(opt, current + 1);
                        }
                    }
                }
            }

            // Move to the next byte
            current += 1;
        }
    }

    fn append_data_to_proc_buffer(&mut self, data_start: usize, data_end: usize) {
        let data_length = data_end - data_start;
        let dst_start = self.process_buffered_size;
        let dst_end = self.process_buffered_size + data_length;
        let dst = &mut self.process_buffer[dst_start .. dst_end];
        dst.copy_from_slice(&self.buffer[data_start .. data_end]);
        self.process_buffered_size += data_length;
    }

    fn copy_buffered_data(&mut self, data_start: usize, data_end: usize) -> Box<[u8]> {
        let data = if self.process_buffered_size > 0 {
            // Copy the data to the process buffer
            self.append_data_to_proc_buffer(data_start, data_end);

            let pbe = self.process_buffered_size;
            self.process_buffered_size = 0;

            &self.process_buffer[0 .. pbe]
        } else {
            &self.buffer[data_start .. data_end]
        };

        Box::from(data)
    }
}

impl Telnet<TcpStream> {
    pub fn read(&mut self) -> io::Result<TelnetEvent> {
        if self.event_queue.is_empty() {
            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer) {
                Ok(size) => {
                    self.buffered_size = size;
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        return Ok(TelnetEvent::NoData);
                    } else {
                        return Err(e);
                    }
                }
            }

            self.process();
        }

        // Return an event
        Ok(
            match self.event_queue.take_event() {
                Some(x) => x,
                None => TelnetEvent::Error("Internal Queue error".to_string())
            }
        )

    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut write_size = 0;

        let mut start = 0;
        for i in 0..data.len() {
            if data[i] == BYTE_IAC {
                self.stream.write(&data[start .. i + 1])?;
                self.stream.write(&[BYTE_IAC])?;
                write_size = write_size + (i + 1 - start);
                start = i + 1;
            }
        }

        if start < data.len() {
            self.stream.write(&data[start .. data.len()])?;
            write_size = write_size + (data.len() - start);
        }

        Ok(write_size)
    }

    pub fn negotiate(&mut self, action: NegotiationAction, opt: TelnetOption) {
        let buf: &[u8] = &[BYTE_IAC, action.to_byte(), opt.to_byte()];
        self.stream.write(buf).expect("Error sending negotiation");
    }

    pub fn subnegotiate(&mut self, opt: TelnetOption, data: &[u8]) {
        let buf: &[u8] = &[BYTE_IAC, BYTE_SB, opt.to_byte()];
        self.stream.write(buf).expect("Error sending subnegotiation (START)");

        self.stream.write(data).expect("Error sending subnegotiation (DATA)");

        let buf: &[u8] = &[BYTE_IAC, BYTE_SE];
        self.stream.write(buf).expect("Error sending subnegotiation (END)");
    }
}

impl Telnet<Async<TcpStream>> {
    pub fn into_inner(self) -> Async<TcpStream> {
        self.stream
    }

    pub async fn read(&mut self) -> io::Result<TelnetEvent> {
        while self.event_queue.is_empty() {
            // Read bytes to the buffer
            match self.stream.read(&mut self.buffer).await {
                Ok(size) => {
                    self.buffered_size = size;
                },
                Err(e) => return Err(e)
            }

            self.process();
        }

        // Return an event
        Ok(
            match self.event_queue.take_event() {
                Some(x) => x,
                None => TelnetEvent::Error("Internal Queue error".to_string())
            }
        )
    }

    pub async fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut write_size = 0;

        let mut start = 0;
        for i in 0..data.len() {
            if data[i] == BYTE_IAC {
                self.stream.write(&data[start .. i + 1]).await?;
                self.stream.write(&[BYTE_IAC]).await?;
                write_size = write_size + (i + 1 - start);
                start = i + 1;
            }
        }

        if start < data.len() {
            self.stream.write(&data[start .. data.len()]).await?;
            write_size = write_size + (data.len() - start);
        }

        Ok(write_size)
    }

    pub async fn negotiate(&mut self, action: NegotiationAction, opt: TelnetOption) {
        let buf: &[u8] = &[BYTE_IAC, action.to_byte(), opt.to_byte()];
        self.stream.write(buf).await.expect("Error sending negotiation");
    }

    pub async fn subnegotiate(&mut self, opt: TelnetOption, data: &[u8]) {
        let buf: &[u8] = &[BYTE_IAC, BYTE_SB, opt.to_byte()];
        self.stream.write(buf).await.expect("Error sending subnegotiation (START)");

        self.stream.write(data).await.expect("Error sending subnegotiation (DATA)");

        let buf: &[u8] = &[BYTE_IAC, BYTE_SE];
        self.stream.write(buf).await.expect("Error sending subnegotiation (END)");
    }
}
