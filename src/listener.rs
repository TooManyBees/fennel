use bcrypt::BcryptError;
use crossbeam_channel::Sender;
use smol::{fs, io, prelude::*, Async};
use std::net::{TcpListener, TcpStream};

use crate::character::PlayerRecord;
use crate::pronoun::Pronoun;
use crate::ConnectionBuilder;
use crate::telnet::{Telnet, TelnetEvent, TelnetOption, NegotiationAction};

#[derive(Debug)]
pub enum LoginError {
    NoName,
    WrongPassword(String),
    IO(io::Error),
    LoadError(LoadError),
    Bcrypt(BcryptError),
}

impl From<io::Error> for LoginError {
    fn from(e: io::Error) -> LoginError {
        LoginError::IO(e)
    }
}

impl From<BcryptError> for LoginError {
    fn from(e: BcryptError) -> LoginError {
        match e {
            BcryptError::Io(e) => LoginError::IO(e),
            _ => LoginError::Bcrypt(e),
        }
    }
}

impl From<LoadError> for LoginError {
    fn from(e: LoadError) -> LoginError {
        LoginError::LoadError(e)
    }
}

#[derive(Debug)]
pub enum LoadError {
    IO(io::Error, String),
    Unparsable(String),
}

async fn read_string(
    stream: &mut Telnet<Async<TcpStream>>,
    timeout: Option<usize>,
) -> Result<String, io::Error> {
    loop {
        match stream.read().await {
            Ok(TelnetEvent::Data(data)) => {
                let v = Vec::from(data);
                let string = String::from_utf8(v)
                    .map_err(|_| std::io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
                return Ok(string.trim().to_string());
            }
            Ok(TelnetEvent::Negotiation(n, o)) => {
                log::debug!("Telnet negotiation {:?} {:?}", n, o);
            }
            Ok(TelnetEvent::Subnegotiation(o, data)) => {
                log::debug!("Telnet subnegotation {:?} {:X?}", o, data);
            }
            Ok(TelnetEvent::Error(s)) => log::debug!("Telnet error: {}", s),
            Ok(TelnetEvent::NoData) | Ok(TelnetEvent::TimedOut) => {},
            Ok(TelnetEvent::UnknownIAC(iac)) => {},
            Err(e) => log::debug!("Telnet io error: {}", e),
        }
    }
}

async fn load_old_character(name: &str) -> Result<Option<PlayerRecord>, LoadError> {
    // FIXME: do this on a SEPARATE thread with its own async reactor dedicated to loading/saving files
    let pfile_path = PlayerRecord::file_path(name);
    match fs::File::open(pfile_path).await {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(LoadError::IO(e, name.to_string())),
        Ok(mut f) => {
            let mut v = Vec::new();
            f.read_to_end(&mut v)
                .await
                .map_err(|e| LoadError::IO(e, name.to_string()))?;
            Ok(Some(serde_json::de::from_slice(&v).map_err(|e| {
                log::error!("Load error {}", e);
                LoadError::Unparsable(name.to_string())
            })?))
        }
    }
}

async fn do_login(stream: &mut Telnet<Async<TcpStream>>) -> Result<PlayerRecord, LoginError> {
    // stream.negotiate(NegotiationAction::Do, TelnetOption::TTYPE).await;
    stream.write(b"What is your name? \xFF\xF9").await?;
    let name = read_string(stream, None).await?;
    if name.is_empty() {
        return Err(LoginError::NoName);
    }
    // TODO: if denied_name? Err(loginError)
    // TODO: if forbidden_name? Err(loginError)

    // stream.subnegotiate(TelnetOption::TTYPE, &[0x01]).await;

    match load_old_character(&name).await? {
        Some(old_character) => do_character_old(stream, old_character).await,
        None => do_character_new(stream, name).await,
    }
}

async fn do_character_old(
    stream: &mut Telnet<Async<TcpStream>>,
    char: PlayerRecord,
) -> Result<PlayerRecord, LoginError> {
    stream.write(b"Password:  \xFF\xF9").await?;
    let password = read_string(stream, None).await?;
    if bcrypt::verify(&password, &char.password())? {
        Ok(char)
    } else {
        Err(LoginError::WrongPassword(char.name()))
    }
}

async fn do_character_new(
    stream: &mut Telnet<Async<TcpStream>>,
    name: String,
) -> Result<PlayerRecord, LoginError> {
    let mut password = None;
    let mut password_confirmed = false;
    let mut pronoun = None;
    stream.write(b"Give us a password. Leading and trailing whitespace will be removed; *interior* whitespace will be preserved. Be careful.\r\nPassword:  \xFF\xF9").await?;
    while password.is_none() {
        let maybe_password = read_string(stream, None).await?;
        if maybe_password.is_empty() {
            stream
                .write(b"You can't leave your password blank. Password:  \xFF\xF9")
                .await?;
            continue;
        }
        let hashed = bcrypt::hash(maybe_password, 4)?;
        password = Some(hashed);
    }
    let password = password.unwrap(); // Unwrapped and immutable

    stream.write(b"Confirm password:  \xFF\xF9").await?;
    while !password_confirmed {
        let maybe_same_password = read_string(stream, None).await?;
        password_confirmed = bcrypt::verify(maybe_same_password, &password)?;
        if !password_confirmed {
            stream
                .write(b"Password doesn't match. Try again:  \xFF\xF9")
                .await?;
        }
    }

    stream
        .write(b"How do we refer to you (it/he/she/they)?  \xFF\xF9")
        .await?;
    while pronoun.is_none() {
        let maybe_pronoun = read_string(stream, None).await?;
        match maybe_pronoun.to_ascii_lowercase().as_str() {
            "it" => pronoun = Some(Pronoun::It),
            "he" => pronoun = Some(Pronoun::He),
            "she" => pronoun = Some(Pronoun::She),
            "they" => pronoun = Some(Pronoun::They),
            _ => {
                stream
                    .write(b"That's not an option we know.\r\nPick again: \xFF\xF9")
                    .await?;
            }
        }
    }
    let pronoun = pronoun.unwrap_or(Pronoun::They); // Unwrapped and immutable

    log::info!("New character {}", name);
    Ok(PlayerRecord::new(name, pronoun, password))
}

pub fn listen(listener: Async<TcpListener>, sender: Sender<(ConnectionBuilder, PlayerRecord)>) {
    smol::block_on(async {
        loop {
            if let Ok((stream, addr)) = listener.accept().await {
                let mut stream = Telnet::from_stream(stream, 160);
                let sender = sender.clone();
                smol::spawn(async move {
                    match do_login(&mut stream).await {
                        Ok(player) => {
                            if let Ok(stream) = stream.into_inner().into_inner() {
                                let connection = ConnectionBuilder::new(stream, addr);
                                // FIXME: sender is a regular crossbeam-channel, not an async channel
                                // is sender.send (potentially blocking) a bad move inside an async
                                // block? Docs for `thread::sleep` say not to use it inside an async
                                // block; maybe this is the same.
                                let _ = sender.send((connection, player));
                            }
                        }
                        Err(e) => {
                            let _ = match e {
                                LoginError::NoName => stream.write(b"No name given, bye!\r\n\xFF\xF9").await,
                                LoginError::WrongPassword(name) => {
                                    log::info!("Failed password attempt on {}", name);
                                    stream.write(b"Wrong password, bye!\r\n\xFF\xF9").await
                                },
                                LoginError::IO(e) => {
                                    log::error!("{}", e);
                                    stream.write(b"Error encountered, bye!\r\n\xFF\xF9").await
                                },
                                LoginError::LoadError(LoadError::Unparsable(name)) => {
                                    log::error!("Error loading pfile {}: unparsable", name);
                                    stream.write(b"Your character couldn't be loaded. Disconnecting for safety.\r\n\xFF\xF9").await
                                }
                                LoginError::LoadError(LoadError::IO(e, name)) => {
                                    log::error!("Error loading pfile {}: {}", name, e);
                                    stream.write(b"Your character couldn't be loaded. Disconnecting for safety.\r\n\xFF\xF9").await
                                }
                                LoginError::Bcrypt(e) => {
                                    log::error!("Error verifying password {}", e);
                                    stream.write(b"Error encountered, bye!\r\n\xFF\xF9").await
                                },
                            };
                            let mut stream = stream.into_inner();
                            let _ = stream.close().await;
                        }
                    }
                }).detach();
            }
        }
    });
}
