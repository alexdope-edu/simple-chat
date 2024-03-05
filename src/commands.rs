use std::{char, str};

// подключена внешняя библиотека https://crates.io/crates/uuid для генерации уникального id

use uuid::Uuid;

use crate::error::Error;

pub const CMD_WHOAMI: &str = "whoami";
pub const CMD_BYE: &str = "bye";

pub enum Command {
    Login(Login),
    Message(Message),
    MessageWithMentions(MessageWithMentions),
    AddUser(AddUser),
    RemoveUser(RemoveUser),
    ShowUsers(ShowUsers),
}

impl Command {
    pub fn new(input: &str) -> Result<Self, Error> {
        let mut chars = input.chars().peekable();

        let command = match chars.peek().ok_or(Error::MissingCommandName)? {
            '%' => {
                chars.next().ok_or(Error::MissingCommandName)?;
                let mut command_name = String::new();

                for current_char in &mut chars {
                    if current_char == ' ' {
                        break;
                    }
                    command_name.push(current_char);
                }

                match command_name.as_str() {
                    Login::COMMAND_NAME => Self::Login(Login::new(chars)?),
                    AddUser::COMMAND_NAME => Self::AddUser(AddUser::new(chars)?),
                    RemoveUser::COMMAND_NAME => Self::RemoveUser(RemoveUser::new(chars)?),
                    ShowUsers::COMMAND_NAME => Self::ShowUsers(ShowUsers::new()),
                    _ => return Err(Error::UnknownCommand),
                }
            }
            '@' => Self::MessageWithMentions(MessageWithMentions::new(chars)?),
            _ => Self::Message(Message::new(chars.collect())),
        };

        Ok(command)
    }
}

pub struct Login {
    pub id: String,
}

impl Login {
    pub const COMMAND_NAME: &'static str = "login";

    pub fn new(input: impl Iterator<Item = char>) -> Result<Self, Error> {
        Ok(Self {
            id: input.collect::<String>().trim().to_string(),
        })
    }
}

pub struct MessageWithMentions {
    user_names: Vec<String>,
    message: String,
}

impl MessageWithMentions {
    pub fn new(input: impl Iterator<Item = char>) -> Result<Self, Error> {
        let mut chars = input.peekable();
        let mut user_names = Vec::new();

        loop {
            if *chars.peek().ok_or(Error::MissingUserName)? != '@' {
                break;
            }

            chars.next().ok_or(Error::MissingUserName)?;
            let mut name = String::new();

            loop {
                let current_char = chars.next().ok_or(Error::MissingUserName)?;

                if current_char == ' ' {
                    break;
                }

                name.push(current_char);
            }

            user_names.push(name);
        }

        Ok(Self {
            user_names,
            message: chars.collect(), // собирает остаток строки
        })
    }
}

pub struct Message {
    pub message: String,
}

impl Message {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

pub struct AddUser {
    id: Uuid,
    kind: UserKind,
}

impl AddUser {
    pub const COMMAND_NAME: &'static str = "add_user";

    pub fn new(mut input: impl Iterator<Item = char>) -> Result<Self, Error> {
        let mut user_id = String::new();

        loop {
            let current_char = input.next().ok_or(Error::MissingArgument)?;

            if current_char == ' ' {
                break;
            }

            user_id.push(current_char);
        }

        let kind: String = input.collect();
        let user_id = Uuid::parse_str(&user_id).map_err(|_| Error::InvalidUuid)?;

        Ok(match kind.as_str() {
            UserKind::NORMAL_KIND => Self {
                id: user_id,
                kind: UserKind::Normal,
            },
            UserKind::ADMIN_KIND => Self {
                id: user_id,
                kind: UserKind::Admin,
            },
            _ => return Err(Error::InvalidUserKind),
        })
    }
}

pub struct RemoveUser {
    id: Uuid,
}

impl RemoveUser {
    pub const COMMAND_NAME: &'static str = "remove_user";

    pub fn new(input: impl Iterator<Item = char>) -> Result<Self, Error> {
        Ok(Self {
            id: Uuid::parse_str(&input.collect::<String>()).map_err(|_| Error::InvalidUuid)?,
        })
    }
}

pub struct ShowUsers;

impl ShowUsers {
    pub const COMMAND_NAME: &'static str = "show_users";

    pub fn new() -> Self {
        Self {}
    }
}

pub enum UserKind {
    Admin,
    Normal,
}

impl UserKind {
    pub const NORMAL_KIND: &'static str = "normal";
    pub const ADMIN_KIND: &'static str = "admin";
}

#[cfg(test)]
mod test {
    use super::Command;

    #[test]
    fn test() {
        let samples = vec![
            "%add_user e634488a-a14e-4166-903c-56ac9f37f8e9 normal",
            "%remove_user e634488a-a14e-4166-903c-56ac9f37f8e9",
            "%login e634488a-a14e-4166-903c-56ac9f37f8e9",
            "%show_users",
            "@Roma @Alex Пацаны, помогите распарсить",
            "Good bye, world!",
        ];
        for sample in samples {
            assert!(Command::new(sample).is_ok(), "{sample}");
        }
    }
}
