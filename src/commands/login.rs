use std::error::Error;

use clap::Args;

use crate::api;
use crate::auth;

#[derive(Args)]
pub struct LoginArgs {
    /// API token (omit to enter interactively)
    #[arg(long)]
    pub token: Option<String>,
}

pub async fn run(
    args: &LoginArgs,
    base_url: &str,
    token_store: &dyn auth::TokenStore,
    prompt_fn: impl FnOnce() -> Result<String, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let token = select_login_token(
        args.token.clone(),
        std::env::var(crate::api::SHORTCUT_API_TOKEN_ENV).ok(),
        prompt_fn,
    )?;

    let sc = api::client_with_token(&token, base_url)?;

    let member = sc.get_current_member_info().send().await.map_err(|e| {
        format!(
            "Authentication failed: {}",
            crate::api::format_api_error(&e)
        )
    })?;

    token_store.store_token(&token)?;

    println!("Logged in as {} (@{})", member.name, member.mention_name);

    Ok(())
}

#[doc(hidden)]
pub fn select_login_token(
    arg_token: Option<String>,
    env_token: Option<String>,
    prompt_fn: impl FnOnce() -> Result<String, Box<dyn Error>>,
) -> Result<String, Box<dyn Error>> {
    if let Some(token) = arg_token {
        return Ok(token);
    }

    if let Some(token) = env_token
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return Ok(token);
    }

    prompt_fn()
}
