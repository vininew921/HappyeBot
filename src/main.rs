use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use actix_web::{web, App, HttpServer};
use happye_bot::{
    auth::{open_browser_and_authenticate, TwitchAuthState},
    error::{TwitchBotError, TwitchBotResult},
    request_endpoints::auth,
    twitch_bot,
};
use tokio::sync::Mutex;
use tracing_subscriber::{self, filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
pub async fn main() -> TwitchBotResult<()> {
    //Initialize environment variables and tracing
    init_env();
    let token_ok = tokio::fs::metadata("token.json").await.is_ok();

    //Twitch credentials
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("Twitch client id must be set");
    let client_secret = std::env::var("TWITCH_CLIENT_SECRET").expect("Twitch secret must be set");
    let port = 42069;

    //Arcs
    let task_shutdown_marker = Arc::new(AtomicBool::new(false));
    let auth_token = Arc::new(Mutex::new(String::from("")));

    //Actix states
    let auth_state = web::Data::new(TwitchAuthState {
        auth_code: auth_token.clone(),
    });

    //Actix server config
    let server = HttpServer::new(move || App::new().app_data(auth_state.clone()).service(auth))
        .bind(("127.0.0.1", port))?
        .disable_signals()
        .run();

    //Handle or stopping the server
    let server_handle = server.handle();

    //Worker tasks
    let server_task = tokio::spawn(server);
    let twitch_bot_task = tokio::spawn(twitch_bot::run_async(
        client_id.clone(),
        client_secret.clone(),
        port,
        token_ok,
        Arc::clone(&auth_token),
        Arc::clone(&task_shutdown_marker),
    ));

    let shutdown = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        // start shutdown of tasks
        let server_stop = server_handle.stop(true);
        task_shutdown_marker.store(true, Ordering::SeqCst);

        // await shutdown of tasks
        server_stop.await;

        Ok::<(), TwitchBotError>(())
    });

    //Open browser to get auth token if token doesn't exist locally
    if !token_ok {
        open_browser_and_authenticate(client_id, port);
    }

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
