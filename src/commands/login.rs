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
    let token = match &args.token {
        Some(t) => t.clone(),
        None => prompt_fn()?,
    };

    let sc = api::client_with_token(&token, base_url)?;

    let member = sc
        .get_current_member_info()
        .send()
        .await
        .map_err(|e| format!("Authentication failed: {e}"))?;

    token_store.store_token(&token)?;

    println!("Logged in as {} (@{})", member.name, member.mention_name);

    Ok(())
}
