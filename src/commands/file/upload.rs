use std::error::Error;
use std::path::PathBuf;

use clap::Args;

use crate::api;
use crate::commands::story::helpers::append_description;
use crate::out_println;
use crate::output::{OutputConfig, format_template};

use super::helpers::{PreparedFile, human_size, markdown_for_value, prepare_files};

#[derive(Args)]
pub struct UploadArgs {
    /// Files to upload
    #[arg(required = true, value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Attach the uploaded files to this story
    #[arg(long)]
    pub story_id: Option<i64>,

    /// Print a ready-to-paste markdown image/link per file instead of the summary
    #[arg(long)]
    pub markdown: bool,

    /// Append markdown images/links for the uploaded files to the story description
    #[arg(long, requires = "story_id")]
    pub embed: bool,
}

/// Options shared by `file upload` and `story file upload`.
pub struct UploadOptions<'a> {
    pub paths: &'a [PathBuf],
    pub story_id: Option<i64>,
    pub markdown: bool,
    pub embed: bool,
}

pub async fn run(
    args: &UploadArgs,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    upload(
        &UploadOptions {
            paths: &args.paths,
            story_id: args.story_id,
            markdown: args.markdown,
            embed: args.embed,
        },
        client,
        out,
    )
    .await
}

pub async fn upload(
    opts: &UploadOptions<'_>,
    client: &api::Client,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if opts.embed && opts.story_id.is_none() {
        return Err("--embed requires --story-id".into());
    }

    let files = prepare_files(opts.paths)?;

    if out.is_dry_run() {
        return dry_run(&files, opts, out);
    }

    let mut uploaded: Vec<serde_json::Value> = Vec::with_capacity(files.len());
    for batch in files.chunks(api::MAX_FILES_PER_UPLOAD) {
        let parts = batch
            .iter()
            .map(|f| {
                let bytes = std::fs::read(&f.path)
                    .map_err(|e| format!("Failed to read file '{}': {e}", f.path.display()))?;
                Ok(api::UploadPart {
                    filename: f.filename.clone(),
                    content_type: f.content_type.clone(),
                    bytes,
                })
            })
            .collect::<Result<Vec<_>, String>>()
            .map_err(|e| with_partial(e, &uploaded))?;

        let mut result = api::upload_files(client, opts.story_id, parts)
            .await
            .map_err(|e| with_partial(format!("Failed to upload files: {e}"), &uploaded))?;
        for file in &mut result {
            let md = markdown_for_value(file);
            if let (Some(obj), Some(md)) = (file.as_object_mut(), md) {
                obj.insert("markdown".into(), serde_json::Value::String(md));
            }
        }
        uploaded.extend(result);
    }

    if opts.embed
        && let Some(story_id) = opts.story_id
    {
        // The uploads already succeeded (and are attached via story_id), so a
        // failed embed must still report them; retrying would create duplicates.
        embed_in_story(story_id, &uploaded, client)
            .await
            .map_err(|e| with_partial(e.to_string(), &uploaded))?;
    }

    print_uploaded(&uploaded, opts, out)
}

/// Add the IDs and markdown snippets of files that were already uploaded to an
/// error message, so a failure after a successful upload never leaves those
/// files unreported and the user can finish (e.g. embed) them by hand.
fn with_partial(msg: String, uploaded: &[serde_json::Value]) -> String {
    if uploaded.is_empty() {
        return msg;
    }
    let ids: Vec<String> = uploaded
        .iter()
        .filter_map(|f| f.get("id").map(|v| v.to_string()))
        .collect();
    let mut out = format!(
        "{msg}\nAlready uploaded before the failure (file IDs): {}",
        ids.join(", ")
    );
    let markdown = markdown_block(uploaded);
    if !markdown.is_empty() {
        out.push_str("\nMarkdown for the uploaded files:\n");
        out.push_str(&markdown);
    }
    out
}

async fn embed_in_story(
    story_id: i64,
    uploaded: &[serde_json::Value],
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let snippets = markdown_block(uploaded);
    if snippets.is_empty() {
        return Err("Uploaded files have no URLs to embed".into());
    }

    let story = client
        .get_story()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Files uploaded, but failed to fetch story {story_id} to embed them: {}",
                api::format_api_error(&e)
            )
        })?;

    let description = append_description(&story.description, &snippets)
        .parse::<api::types::UpdateStoryDescription>()
        .map_err(|e| {
            format!("Files uploaded, but the new description for story {story_id} is invalid: {e}")
        })?;

    client
        .update_story()
        .story_public_id(story_id)
        .body_map(|b| b.description(Some(description)))
        .send()
        .await
        .map_err(|e| {
            format!(
                "Files uploaded, but failed to update story {story_id} description: {}",
                api::format_api_error(&e)
            )
        })?;
    Ok(())
}

/// Markdown snippets for all files, one paragraph each.
fn markdown_block(uploaded: &[serde_json::Value]) -> String {
    uploaded
        .iter()
        .filter_map(|f| f.get("markdown").and_then(|m| m.as_str()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn print_uploaded(
    uploaded: &[serde_json::Value],
    opts: &UploadOptions<'_>,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    if out.is_machine_readable() {
        out_println!(out, "{}", serde_json::to_string_pretty(uploaded)?);
        return Ok(());
    }
    if let Some(template) = out.format_template() {
        for file in uploaded {
            out_println!(out, "{}", format_template(template, file)?);
        }
        return Ok(());
    }
    if opts.markdown {
        for file in uploaded {
            if let Some(md) = file.get("markdown").and_then(|m| m.as_str()) {
                out_println!(out, "{md}");
            }
        }
        return Ok(());
    }
    if out.is_quiet() {
        for file in uploaded {
            if let Some(id) = file.get("id") {
                out_println!(out, "{id}");
            }
        }
        return Ok(());
    }

    for file in uploaded {
        let id = file.get("id").map(|v| v.to_string()).unwrap_or_default();
        let name = file
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let size = file
            .get("size")
            .and_then(|v| v.as_u64())
            .map(|s| format!(" ({})", human_size(s)))
            .unwrap_or_default();
        let url = file
            .get("url")
            .and_then(|v| v.as_str())
            .map(|u| format!(" -> {u}"))
            .unwrap_or_default();
        out_println!(out, "Uploaded file #{id} {name}{size}{url}");
    }
    if let Some(story_id) = opts.story_id {
        let verb = if opts.embed {
            "Attached to and embedded in"
        } else {
            "Attached to"
        };
        out_println!(out, "{verb} story {story_id}");
    }
    Ok(())
}

fn dry_run(
    files: &[PreparedFile],
    opts: &UploadOptions<'_>,
    out: &OutputConfig,
) -> Result<(), Box<dyn Error>> {
    for batch in files.chunks(api::MAX_FILES_PER_UPLOAD) {
        let parts: Vec<serde_json::Value> = batch
            .iter()
            .enumerate()
            .map(|(i, f)| {
                serde_json::json!({
                    "field": format!("file{i}"),
                    "path": f.path.display().to_string(),
                    "filename": f.filename,
                    "content_type": f.content_type,
                    "size": f.size,
                })
            })
            .collect();
        let body = serde_json::json!({
            "story_id": opts.story_id,
            "files": parts,
        });
        out.dry_run_request("POST", "/api/v3/files (multipart/form-data)", Some(&body))?;
    }
    if opts.embed
        && let Some(story_id) = opts.story_id
    {
        out.dry_run_request::<serde_json::Value>(
            "GET",
            &format!("/api/v3/stories/{story_id}"),
            None,
        )?;
        let body = serde_json::json!({
            "description": "<existing description>\n\n<markdown for each uploaded file>"
        });
        out.dry_run_request("PUT", &format!("/api/v3/stories/{story_id}"), Some(&body))?;
    }
    Ok(())
}
