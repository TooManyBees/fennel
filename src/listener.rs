use smol::{io, Async, prelude::*};
use std::net::{TcpListener, TcpStream, SocketAddr};
use crossbeam_channel::Sender;
use bcrypt::BcryptError;

use crate::character::Character;
use crate::pronoun::Pronoun;
use crate::Connection;

pub enum LoginError {
    NoName,
    WrongPassword,
    IO(io::Error),
    Unknown,
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
            _ => LoginError::Unknown,
        }
    }
}

async fn read_string(buf: &mut [u8], stream: &mut Async<TcpStream>, max_len: usize, timeout: Option<usize>) -> Result<String, io::Error> {
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

async fn load_old_character(name: &str) -> Option<Character> {
    None
}

async fn do_login(stream: &mut Async<TcpStream>) -> Result<Character, LoginError> {
    let mut buf = [0u8; 160];

    stream.write_all(b"What is your name? ").await?;
    let name = read_string(&mut buf, stream, 32, None).await?;
    if name.is_empty() {
        return Err(LoginError::NoName);
    }
    // TODO: if denied_name? Err(loginError)
    // TODO: if forbidden_name? Err(loginError)

    return match load_old_character(&name).await {
        Some(old_character) => do_character_old(stream, buf, old_character).await,
        None => do_character_new(stream, buf, name).await,
    }
}

async fn do_character_old(stream: &mut Async<TcpStream>, mut buf: [u8; 160], char: Character) -> Result<Character, LoginError> {
    stream.write_all(b"Password: ").await?;
    let password = read_string(&mut buf, stream, 160, None).await?;
    if bcrypt::verify(char.password(), &password)? {
        Ok(char)
    } else {
        Err(LoginError::WrongPassword)
    }
}

async fn do_character_new(stream: &mut Async<TcpStream>, mut buf: [u8; 160], name: String) -> Result<Character, LoginError> {
    let mut password = None;
    let mut password_confirmed = false;
    let mut pronoun = None;
    stream.write_all(b"Password: ").await?;
    while password.is_none() {
        let maybe_password = read_string(&mut buf, stream, 160, None).await?;
        // TODO: test for whitespace inside password too
        if maybe_password.is_empty() {
            stream.write_all(b"You can't leave your password blank. Password: ").await?;
            continue;
        }
        let hashed = bcrypt::hash(maybe_password, 4)?;
        password = Some(hashed);
    }
    let password = password.unwrap(); // Unwrapped and immutable

    stream.write_all(b"Confirm password: ").await?;
    while !password_confirmed {
        let maybe_same_password = read_string(&mut buf, stream, 160, None).await?;
        password_confirmed = bcrypt::verify(maybe_same_password, &password)?;
        if !password_confirmed {
            stream.write_all(b"Password doesn't match. Try again: ").await?;
        }
    }

    stream.write_all(b"How do we refer to you (it/he/she/they)?").await?;
    while pronoun.is_none() {
        let maybe_pronoun = read_string(&mut buf, stream, 32, None).await?;
        match maybe_pronoun.to_ascii_lowercase().as_str() {
            "it" => pronoun = Some(Pronoun::It),
            "he" => pronoun = Some(Pronoun::He),
            "she" => pronoun = Some(Pronoun::She),
            "they" => pronoun = Some(Pronoun::They),
            _ => stream.write_all(b"That's not an option we know.\r\nPick again: ").await?,
        }
    }
    let pronoun = pronoun.unwrap_or(Pronoun::They); // Unwrapped and immutable

    Ok(Character::new(name, pronoun, password))
}

pub fn listen(listener: Async<TcpListener>, sender: Sender<(Connection, Character)>) {
    smol::block_on(async {
        loop {
            if let Ok((mut stream, addr)) = listener.accept().await {
                let sender = sender.clone();
                smol::spawn(async move {
                    match do_login(&mut stream).await {
                        Ok(char) => {
                            if let Ok(stream) = stream.into_inner() {
                                let connection = Connection::new(stream, addr);
                                let _ = sender.send((connection, char));
                            }
                        }
                        Err(e) => {
                            let _ = match e {
                                LoginError::NoName => stream.write_all(b"No name given, bye!").await,
                                LoginError::WrongPassword => stream.write_all(b"Wrong password, bye!").await,
                                LoginError::IO(_) => stream.write_all(b"Error encountered, bye!").await,
                                LoginError::Unknown => stream.write_all(b"Error encountered, bye!").await,
                            };
                            let _ = stream.close().await;
                        }
                    }
                }).detach();
            }
        }
    });
}
