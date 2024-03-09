use std::collections::HashMap;

use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Command {
    pub response: String,
    pub tag_user: bool,
}

impl Command {
    pub fn new(response: String, tag_user: bool) -> Self {
        Self { response, tag_user }
    }
}

pub fn get_command(command: String) -> Option<Command> {
    COMMANDS.get(&command.as_str()).cloned()
}

//Add a "!hi" command for testing
lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Command> = {
        let mut commands = HashMap::new();
        commands.insert("!hi", Command::new("Salve @<user>".to_string(), true));

        commands.insert(
            "!github",
            Command::new("https://github.com/vininew921".to_string(), false),
        );

        commands
    };
}
