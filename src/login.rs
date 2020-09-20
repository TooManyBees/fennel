use std::mem;

use crate::Connection;
use crate::character::Character;
use crate::pronoun::Pronoun;
use crate::login::ConnectState::LoginExisting;

#[derive(Debug)]
pub struct CharacterCreation {
    name: String,
    password: Option<String>,
    password_confirmed: bool,
    pronoun: Option<Pronoun>,
}

impl CharacterCreation {
    pub fn new(name: String) -> CharacterCreation {
        CharacterCreation {
            name,
            password: None,
            password_confirmed: false,
            pronoun: None,
        }
    }
}

pub enum ConnectState {
    GetName,
    LoginExisting(String),
    Create(CharacterCreation),
    Indeterminate,
}

pub enum LoginError {
    IndeterminateState,

}

pub fn login(connection: &mut Connection) -> Result<(), LoginError> {
    if let Some(input) = connection.input.take() {
        let mut connect_state = ConnectState::Indeterminate;
        mem::swap(&mut connection.connect_state, &mut connect_state);
        match connect_state {
            ConnectState::GetName => {
                let name = input.trim().to_string();
                // TODO: look up char; are we logging in, or creating anew?
                connection.connect_state = ConnectState::Create(CharacterCreation::new(name));
                connection.write("Password: ");
                Ok(())
            }
            ConnectState::LoginExisting(name) => {
                unimplemented!()
            }
            ConnectState::Create(mut char) => {
                if char.password.is_none() {
                    let password = input.trim().to_string();
                    char.password = Some(password);
                    // TODO: bcrypt this
                    connection.connect_state = ConnectState::Create(char);
                    connection.write("Confirm password: ");
                    return Ok(());
                }

                if !char.password_confirmed {
                    let password = input.trim().to_string();
                    char.password_confirmed = char.password == Some(password);
                    if char.password_confirmed {
                        connection.write("How do we refer to you (it/he/she/they)? ");
                    } else {
                        connection.write("Passwords don't match.\r\nRetype: ");
                    }
                    connection.connect_state = ConnectState::Create(char);
                    return Ok(());
                }

                if char.pronoun.is_none() {
                    match input.to_ascii_lowercase().trim() {
                        "it" => char.pronoun = Some(Pronoun::It),
                        "he" => char.pronoun = Some(Pronoun::He),
                        "she" => char.pronoun = Some(Pronoun::She),
                        "they" => char.pronoun = Some(Pronoun::They),
                        _ => {
                            connection.write("That's not an option we know.\r\nPick again: ");
                            connection.connect_state = ConnectState::Create(char);
                            return Ok(());
                        },
                    }
                }

                let new_char = Character::new(char.name, char.pronoun.unwrap_or(Pronoun::It));
                println!("{:?}", new_char);
                connection.character = Some(new_char);
                Ok(())
            }
            ConnectState::Indeterminate => Err(LoginError::IndeterminateState),
        }
    }
    Ok(())
}