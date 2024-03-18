use frankenstein::{Message, MessageEntityType};

pub enum Command {
    Start,
    Help,
    Info,
    MkDir,
}

impl TryFrom<Message> for Command {
    type Error = String;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let text_command = message
            .text
            .ok_or_else(|| "No text in message".to_string())?;

        let entity = message
            .entities
            .map(|e| e.first().cloned())
            .flatten()
            .ok_or_else(|| "No entities in message".to_string())?;

        if entity.type_field != MessageEntityType::BotCommand {
            return Err("No bot command in message".to_string());
        }

        let offset = entity.offset as usize;
        let length = entity.length as usize;
        let command = &text_command[offset..offset + length];

        match command {
            "/start" => Ok(Command::Start),
            "/help" => Ok(Command::Help),
            "/info" => Ok(Command::Info),
            "/mkdir" => Ok(Command::MkDir),
            _ => Err("Unknown command".to_string()),
        }
    }
}
