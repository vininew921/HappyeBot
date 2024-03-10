use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Command {
    pub response: String,
    pub tag_user: bool,
    pub timeout_seconds: u32,
    last_called: Option<DateTime<Utc>>,
}

impl Command {
    pub fn new(response: String, tag_user: bool, timeout_seconds: u32) -> Self {
        Self {
            response,
            tag_user,
            timeout_seconds,
            last_called: None,
        }
    }

    pub fn update_last_called(&mut self) {
        self.last_called = Some(Utc::now());
    }
}

pub fn get_command(command_text: String) -> Option<Command> {
    let mut map = COMMANDS.lock().unwrap();

    if let Some(command) = map.get_mut(command_text.as_str()) {
        if command.timeout_seconds == 0
            || command.last_called.is_none()
            || command.last_called.unwrap()
                + Duration::try_seconds(command.timeout_seconds.into()).unwrap()
                <= Utc::now()
        {
            command.last_called = Some(Utc::now());

            return Some(command.clone());
        }

        tracing::info!("Command {} is still timed out", command_text);
    }

    None
}

//Add a "!hi" command for testing
lazy_static! {
    static ref COMMANDS: Mutex<HashMap<&'static str, Command>> = {
        let mut commands = HashMap::new();
        commands.insert("!hi", Command::new("Salve @<user>".to_string(), true, 10));

        commands.insert(
            "!github",
            Command::new("https://github.com/vininew921".to_string(), false, 60),
        );

        Mutex::new(commands)
    };
}
