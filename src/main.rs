use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use happye_bot::{
    browser,
    error::{TwitchBotError, TwitchBotResult},
    request_endpoints::{self, BotAuthState},
    twitch_bot,
};
use tokio::sync::Mutex;
use tracing_subscriber::{self, filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
pub async fn main() -> TwitchBotResult<()> {
    //Initialize environment variables and tracing
    init_env();

    //Twitch credentials
    let twitch_id = std::env::var("TWITCH_CLIENT_ID").expect("Twitch client id must be set");
    let twitch_secret = std::env::var("TWITCH_CLIENT_SECRET").expect("Twitch secret must be set");
    let spotify_id = std::env::var("SPOTIFY_CLIENT_ID").expect("Spotify client id must be set");
    let spotify_secret = std::env::var("SPOTIFY_SECRET").expect("Spotify secret must be set");
    let port = 42069;

    //Arcs
    let twitch_auth_token = Arc::new(Mutex::new(String::from("")));
    let spotify_auth_token = Arc::new(Mutex::new(String::from("")));

    //Actix states
    let bot_auth_state = web::Data::new(BotAuthState {
        twitch_auth_code: twitch_auth_token.clone(),
        spotify_auth_code: spotify_auth_token.clone(),
    });

    //Actix server config
    let server = HttpServer::new(move || {
        App::new()
            .app_data(bot_auth_state.clone())
            .service(request_endpoints::auth)
            .service(request_endpoints::spotify_auth)
    })
    .bind(("127.0.0.1", port))?
    .disable_signals()
    .run();

    //Handle or stopping the server
    let server_handle = server.handle();

    //Worker tasks
    let server_task = tokio::spawn(server);
    let twitch_bot_task = tokio::spawn(twitch_bot::run_async(
        twitch_id.clone(),
        twitch_secret,
        spotify_id.clone(),
        spotify_secret,
        port,
        Arc::clone(&twitch_auth_token),
        Arc::clone(&spotify_auth_token),
    ));

    let shutdown = tokio::spawn(async move {
        tokio::signal::ctrl_c().await?;
        let server_stop = server_handle.stop(true);
        server_stop.await;
        Ok::<(), TwitchBotError>(())
    });

    //Open browser to get twitch and spotify token if it doesn't exist locally
    browser::open_browser_and_authenticate_twitch(twitch_id, port).await;
    browser::open_browser_and_authenticate_spotify(spotify_id, port).await;

    //Join all tasks and wait for the shutdown signal
    tokio::try_join!(server_task, twitch_bot_task, shutdown)
        .expect("unable to join tasks")
        .0?;

    Ok(())
}

fn init_env() {
    dotenvy::dotenv().ok();

    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(stdout_log.with_filter(filter::LevelFilter::INFO))
        .init();
}
