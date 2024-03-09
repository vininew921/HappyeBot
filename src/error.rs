#[derive(Debug, thiserror::Error)]
pub enum TwitchBotError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ResponseError(#[from] reqwest::Error),

    #[error(transparent)]
    TwitchIrcValidationError(#[from] twitch_irc::validate::Error),

    #[error("Could not load Twitch Token")]
    TwitchTokenLoadError(),

    #[error("Could not update Twitch Token")]
    TwitchTokenUpdateError(),
}

pub type TwitchBotResult<T, E = TwitchBotError> = anyhow::Result<T, E>;
