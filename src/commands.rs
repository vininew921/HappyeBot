use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Command {
    pub response: String,
    pub timeout_seconds: u32,
    pub api_call: Option<String>,
    last_called: Option<DateTime<Utc>>,
}

impl Command {
    pub fn new(response: String, timeout_seconds: u32, api_call: Option<String>) -> Self {
        Self {
            response,
            timeout_seconds,
            api_call,
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

        commands.insert(
            "!github",
            Command::new("https://github.com/vininew921".to_string(), 60, None),
        );

        commands.insert(
            "!sr",
            Command::new(
                "Musica <song> adicionada a fila".to_string(),
                30,
                Some("play_track".to_string()),
            ),
        );

        Mutex::new(commands)
    };
}
