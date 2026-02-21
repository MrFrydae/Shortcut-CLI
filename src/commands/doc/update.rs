use std::error::Error;
use std::path::PathBuf;

use clap::Args;

use crate::api;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The document UUID
    #[arg(long)]
    pub id: String,

    /// The new title
    #[arg(long)]
    pub title: Option<String>,

    /// The new content (inline string)
    #[arg(long, conflicts_with = "content_file")]
    pub content: Option<String>,

    /// Read new content from a file
    #[arg(long)]
    pub content_file: Option<PathBuf>,

    /// Content format: markdown or html
    #[arg(long)]
    pub content_format: Option<String>,
}

pub async fn run(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = args
        .id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let content = if let Some(path) = &args.content_file {
        Some(
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?,
        )
    } else {
        args.content.clone()
    };

    let title = args
        .title
        .as_ref()
        .map(|t| t.parse::<api::types::UpdateDocTitle>())
        .transpose()
        .map_err(|e| format!("Invalid title: {e}"))?;

    let content_format = args
        .content_format
        .as_ref()
        .map(|f| f.parse::<api::types::UpdateDocContentFormat>())
        .transpose()
        .map_err(|e| format!("Invalid content format: {e}"))?;

    let doc = client
        .update_doc()
        .doc_public_id(doc_id)
        .body_map(|mut b| {
            if let Some(title) = title {
                b = b.title(Some(title));
            }
            if let Some(c) = content {
                b = b.content(Some(c));
            }
            if let Some(fmt) = content_format {
                b = b.content_format(Some(fmt));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update document: {e}"))?;

    let title = doc.title.as_deref().unwrap_or("(untitled)");
    println!("Updated document {} - {}", doc.id, title);
    Ok(())
}
