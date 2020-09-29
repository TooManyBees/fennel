use bcrypt::BcryptError;
use crossbeam_channel::Sender;
use smol::{fs, io, prelude::*, Async};
use std::net::{TcpListener, TcpStream};

use crate::character::PlayerRecord;
use crate::pronoun::Pronoun;
use crate::ConnectionBuilder;

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
    buf: &mut [u8],
    stream: &mut Async<TcpStream>,
    max_len: usize,
    timeout: Option<usize>,
) -> Result<String, io::Error> {
    let bytes_read = stream.read(buf).await?;
    if bytes_read == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Read 0 bytes"));
    }
    // TODO: test for max_len
    // TODO: test for timeout
    let string = String::from_utf8(buf[..bytes_read].to_vec())
        .map_err(|_| std::io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
    let string = string.trim().to_string();
    Ok(string)
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

async fn do_login(stream: &mut Async<TcpStream>) -> Result<PlayerRecord, LoginError> {
    let mut buf = [0u8; 160];

    stream.write_all(b"What is your name? \xFF\xF9").await?;
    let name = read_string(&mut buf, stream, 32, None).await?;
    if name.is_empty() {
        return Err(LoginError::NoName);
    }
    // TODO: if denied_name? Err(loginError)
    // TODO: if forbidden_name? Err(loginError)

    return match load_old_character(&name).await? {
        Some(old_character) => do_character_old(stream, buf, old_character).await,
        None => do_character_new(stream, buf, name).await,
    };
}

async fn do_character_old(
    stream: &mut Async<TcpStream>,
    mut buf: [u8; 160],
    char: PlayerRecord,
) -> Result<PlayerRecord, LoginError> {
    stream.write_all(b"Password: \xFF\xF9").await?;
    let password = read_string(&mut buf, stream, 160, None).await?;
    if bcrypt::verify(&password, &char.password())? {
        Ok(char)
    } else {
        Err(LoginError::WrongPassword(char.name()))
    }
}

async fn do_character_new(
    stream: &mut Async<TcpStream>,
    mut buf: [u8; 160],
    name: String,
) -> Result<PlayerRecord, LoginError> {
    let mut password = None;
    let mut password_confirmed = false;
    let mut pronoun = None;
    stream.write_all(b"Give us a password. Leading and trailing whitespace will be removed; *interior* whitespace will be preserved. Be careful.\r\nPassword: \xFF\xF9").await?;
    while password.is_none() {
        let maybe_password = read_string(&mut buf, stream, 160, None).await?;
        if maybe_password.is_empty() {
            stream
                .write_all(b"You can't leave your password blank. Password: \xFF\xF9")
                .await?;
            continue;
        }
        let hashed = bcrypt::hash(maybe_password, 4)?;
        password = Some(hashed);
    }
    let password = password.unwrap(); // Unwrapped and immutable

    stream.write_all(b"Confirm password: \xFF\xF9").await?;
    while !password_confirmed {
        let maybe_same_password = read_string(&mut buf, stream, 160, None).await?;
        password_confirmed = bcrypt::verify(maybe_same_password, &password)?;
        if !password_confirmed {
            stream
                .write_all(b"Password doesn't match. Try again: \xFF\xF9")
                .await?;
        }
    }

    stream
        .write_all(b"How do we refer to you (it/he/she/they)? \xFF\xF9")
        .await?;
    while pronoun.is_none() {
        let maybe_pronoun = read_string(&mut buf, stream, 32, None).await?;
        match maybe_pronoun.to_ascii_lowercase().as_str() {
            "it" => pronoun = Some(Pronoun::It),
            "he" => pronoun = Some(Pronoun::He),
            "she" => pronoun = Some(Pronoun::She),
            "they" => pronoun = Some(Pronoun::They),
            _ => {
                stream
                    .write_all(b"That's not an option we know.\r\nPick again: \xFF\xF9")
                    .await?
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
            if let Ok((mut stream, addr)) = listener.accept().await {
                let sender = sender.clone();
                smol::spawn(async move {
                    match do_login(&mut stream).await {
                        Ok(player) => {
                            if let Ok(stream) = stream.into_inner() {
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
                                LoginError::NoName => stream.write_all(b"No name given, bye!\r\n\xFF\xF9").await,
                                LoginError::WrongPassword(name) => {
                                    log::info!("Failed password attempt on {}", name);
                                    stream.write_all(b"Wrong password, bye!\r\n\xFF\xF9").await
                                },
                                LoginError::IO(e) => {
                                    log::error!("{}", e);
                                    stream.write_all(b"Error encountered, bye!\r\n\xFF\xF9").await
                                },
                                LoginError::LoadError(LoadError::Unparsable(name)) => {
                                    log::error!("Error loading pfile {}: unparsable", name);
                                    stream.write_all(b"Your character couldn't be loaded. Disconnecting for safety.\r\n\xFF\xF9").await
                                }
                                LoginError::LoadError(LoadError::IO(e, name)) => {
                                    log::error!("Error loading pfile {}: {}", name, e);
                                    stream.write_all(b"Your character couldn't be loaded. Disconnecting for safety.\r\n\xFF\xF9").await
                                }
                                LoginError::Bcrypt(e) => {
                                    log::error!("Error verifying password {}", e);
                                    stream.write_all(b"Error encountered, bye!\r\n\xFF\xF9").await
                                },
                            };
                            let _ = stream.close().await;
                        }
                    }
                }).detach();
            }
        }
    });
}
