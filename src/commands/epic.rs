use std::error::Error;

use clap::Args;

use crate::api;

#[derive(Args)]
pub struct EpicArgs {
    /// List all epics
    #[arg(long)]
    pub list: bool,

    /// Include epic descriptions in output
    #[arg(long, visible_alias = "descriptions")]
    pub desc: bool,
}

pub async fn run(args: &EpicArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if args.list {
        let mut req = client.list_epics();
        if args.desc {
            req = req.includes_description(true);
        }
        let epics = req
            .send()
            .await
            .map_err(|e| format!("Failed to list epics: {e}"))?;
        for epic in epics.iter() {
            println!("{} - {}", epic.id, epic.name);
            if args.desc
                && let Some(desc) = &epic.description
            {
                println!("  {}", desc);
            }
        }
    }
    Ok(())
}
