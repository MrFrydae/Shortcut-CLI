use std::error::Error;
use std::path::PathBuf;

use clap::Args;

use crate::api;
use crate::out_println;
use crate::output::OutputConfig;

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

pub async fn run(
    args: &CreateArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
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

    if out.is_dry_run() {
        let mut body = serde_json::json!({
            "title": args.title,
            "content": content,
        });
        if let Some(fmt) = &args.content_format {
            body["content_format"] = serde_json::json!(fmt);
        }
        return out.dry_run_request("POST", "/api/v3/docs", Some(&body));
    }

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

    if out.is_json() {
        let json = serde_json::to_string_pretty(&*doc)?;
        out.write_str(format_args!("{json}"))?;
        return Ok(());
    }

    if out.is_quiet() {
        out_println!(out, "{}", doc.id);
        return Ok(());
    }

    let title = doc.title.as_deref().unwrap_or("(untitled)");
    out_println!(out, "Created document {} - {}", doc.id, title);
    Ok(())
}
