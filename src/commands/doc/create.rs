use std::error::Error;
use std::path::PathBuf;

use clap::Args;

use crate::api;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The document title
    #[arg(long)]
    pub title: String,

    /// The document content (inline string)
    #[arg(long, conflicts_with = "content_file")]
    pub content: Option<String>,

    /// Read content from a file
    #[arg(long)]
    pub content_file: Option<PathBuf>,

    /// Content format: markdown or html (default: markdown)
    #[arg(long)]
    pub content_format: Option<String>,
}

pub async fn run(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let content = if let Some(path) = &args.content_file {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?
    } else if let Some(c) = &args.content {
        c.clone()
    } else {
        return Err("Either --content or --content-file is required".into());
    };

    let content_format = args
        .content_format
        .as_ref()
        .map(|f| f.parse::<api::types::CreateDocContentFormat>())
        .transpose()
        .map_err(|e| format!("Invalid content format: {e}"))?;

    let doc = client
        .create_doc()
        .body_map(|mut b| {
            b = b.title(args.title.clone()).content(content);
            if let Some(fmt) = content_format {
                b = b.content_format(Some(fmt));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create document: {e}"))?;

    let title = doc.title.as_deref().unwrap_or("(untitled)");
    println!("Created document {} - {}", doc.id, title);
    Ok(())
}
