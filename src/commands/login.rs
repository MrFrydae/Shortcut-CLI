use std::error::Error;

use clap::Args;

use crate::api;
use crate::auth;

#[derive(Args)]
pub struct LoginArgs {
    /// API token (omit to enter interactively)
    #[arg(long)]
    token: Option<String>,
}

pub async fn run(args: LoginArgs) -> Result<(), Box<dyn Error>> {
    let token = match args.token {
        Some(t) => t,
        None => rpassword::prompt_password("Shortcut API token: ")?,
    };

    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "Shortcut-Token",
                reqwest::header::HeaderValue::from_str(&token)?,
            );
            headers
        })
        .build()?;

    let sc = api::Client::new_with_client(api::BASE_URL, client);

    let member = sc
        .get_current_member_info()
        .send()
        .await
        .map_err(|e| format!("Authentication failed: {e}"))?;

    auth::store_token(&token)?;

    println!("Logged in as {} (@{})", member.name, member.mention_name);

    Ok(())
}
