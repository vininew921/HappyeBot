use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use actix_web::{get, web, App, HttpServer};
use happye_bot::auth::bot_auth::{
    get_user_access_token, HappyeBotTokenStorage, TwitchAuthState, TwitchUserAuthResponse,
};
use tokio::{sync::Mutex, time::sleep};
use tracing_subscriber::{self, filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use twitch_irc::{
    login::RefreshingLoginCredentials, ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    //Read credentials from .env file
    init_env();

    //Twitch credentials
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("Twitch client id must be set");
    let client_secret = std::env::var("TWITCH_CLIENT_SECRET").expect("Twitch secret must be set");
    let port = 42069;

    //Arcs
    let task_shutdown_marker = Arc::new(AtomicBool::new(false));
    let auth_token = Arc::new(Mutex::new(String::from("")));

    let auth_state = web::Data::new(TwitchAuthState {
        auth_code: auth_token.clone(),
    });

    //initialize actix web
    let server = HttpServer::new(move || App::new().app_data(auth_state.clone()).service(auth))
        .bind(("127.0.0.1", port))?
        .disable_signals()
        .run();

    let server_handle = server.handle();
    let server_task = tokio::spawn(server);
    let twitch_bot_task = tokio::spawn(run_twitch_bot(
        client_id.clone(),
        client_secret.clone(),
        port,
        Arc::clone(&auth_token),
        Arc::clone(&task_shutdown_marker),
    ));

    let shutdown = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await.unwrap();

        // start shutdown of tasks
        let server_stop = server_handle.stop(true);
        task_shutdown_marker.store(true, Ordering::SeqCst);

        // await shutdown of tasks
        server_stop.await;
    });

    //Bot scopes
    let scopes = vec!["chat:edit", "chat:read"].join("+").replace(":", "%3A");

    //Open browser to get auth token
    let _ = open::that(format!("https://id.twitch.tv/oauth2/authorize?response_type=code&client_id={}&redirect_uri=http://localhost:{}/auth&scope={}", client_id, port, scopes));

    //Join all tasks and wait for the shutdown signal
    let _ = tokio::try_join!(server_task, twitch_bot_task, shutdown).expect("unable to join tasks");

    Ok(())
}

#[get("/auth")]
async fn auth(
    info: web::Query<TwitchUserAuthResponse>,
    auth_state: web::Data<TwitchAuthState>,
) -> String {
    let mut auth_token = auth_state.auth_code.lock().await;
    *auth_token = info.code.clone();

    "You can close this now 🎉".into()
}

async fn run_twitch_bot(
    client_id: String,
    client_secret: String,
    port: u16,
    auth_token: Arc<Mutex<String>>,
    shutdown_marker: Arc<AtomicBool>,
) -> std::io::Result<()> {
    while auth_token.lock().await.as_str() == "" {
        if shutdown_marker.load(Ordering::SeqCst) {
            return Ok(());
        }

        tracing::info!("Waiting for auth token");
        sleep(Duration::from_millis(2000)).await;
    }

    let auth_token_value = auth_token.lock().await.clone();

    let storage = HappyeBotTokenStorage {
        token: Some(
            get_user_access_token(
                client_id.clone(),
                client_secret.clone(),
                auth_token_value,
                port,
            )
            .await,
        ),
    };

    let credentials = RefreshingLoginCredentials::init(client_id, client_secret, storage);

    let twitch_config = ClientConfig::new_simple(credentials);
    let (mut incoming_messages, client) = TwitchIRCClient::<
        SecureTCPTransport,
        RefreshingLoginCredentials<HappyeBotTokenStorage>,
    >::new(twitch_config);

    client.join("vynny_".to_owned()).unwrap();

    client
        .say("vynny_".to_owned(), "Salve galerinha xdd !".to_owned())
        .await
        .unwrap();

    client
        .say(
            "vynny_".to_owned(),
            "AlienGathering xddwicked Suske THIS borpaSpin".to_owned(),
        )
        .await
        .unwrap();

    while let Some(message) = incoming_messages.recv().await {
        if shutdown_marker.load(Ordering::SeqCst) {
            break;
        }

        println!("Received message: {:?}", message);
    }

    Ok(())
}

fn init_env() {
    dotenvy::dotenv().ok();

    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(stdout_log.with_filter(filter::LevelFilter::INFO))
        .init();
}
