use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{sync::Mutex, time::sleep};
use twitch_irc::{
    login::RefreshingLoginCredentials, ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::auth::{get_user_access_token_async, TwitchTokenStorage};

pub async fn run_async(
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
        sleep(Duration::from_millis(1000)).await;
    }

    let auth_token_value = auth_token.lock().await.clone();
    let token = get_user_access_token_async(
        client_id.clone(),
        client_secret.clone(),
        auth_token_value,
        port,
    )
    .await;

    let storage = TwitchTokenStorage { token };

    let credentials = RefreshingLoginCredentials::init(client_id, client_secret, storage);

    let twitch_config = ClientConfig::new_simple(credentials);
    let (mut incoming_messages, client) = TwitchIRCClient::<
        SecureTCPTransport,
        RefreshingLoginCredentials<TwitchTokenStorage>,
    >::new(twitch_config);

    client.join("vynny_".to_owned()).unwrap();

    while let Some(message) = incoming_messages.recv().await {
        if shutdown_marker.load(Ordering::SeqCst) {
            break;
        }

        tracing::info!("Received message: {:?}", message);
    }

    Ok(())
}
