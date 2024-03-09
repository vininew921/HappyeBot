use actix_web::{get, web};

use crate::auth::{TwitchAuthState, TwitchUserAuthResponse};

#[get("/auth")]
async fn auth(
    info: web::Query<TwitchUserAuthResponse>,
    auth_state: web::Data<TwitchAuthState>,
) -> String {
    let mut auth_token = auth_state.auth_code.lock().await;
    *auth_token = info.code.clone();

    "You can close this now 🎉".into()
}
