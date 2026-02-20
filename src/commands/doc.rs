use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct DocArgs {
    #[command(subcommand)]
    pub action: DocAction,
}

#[derive(Subcommand)]
pub enum DocAction {
    /// List all documents
    List,
    /// Create a new document
    Create(Box<CreateArgs>),
    /// Get a document by ID
    Get {
        /// The document UUID
        #[arg(long)]
        id: String,
    },
    /// Update a document
    Update(Box<UpdateArgs>),
    /// Delete a document
    Delete {
        /// The document UUID
        #[arg(long)]
        id: String,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Link a document to an epic
    Link {
        /// The document UUID
        #[arg(long)]
        doc_id: String,
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
    },
    /// Unlink a document from an epic
    Unlink {
        /// The document UUID
        #[arg(long)]
        doc_id: String,
        /// The epic ID
        #[arg(long)]
        epic_id: i64,
    },
    /// List epics linked to a document
    Epics {
        /// The document UUID
        #[arg(long)]
        id: String,
    },
}

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

pub async fn run(args: &DocArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        DocAction::List => run_list(client).await,
        DocAction::Create(create_args) => run_create(create_args, client).await,
        DocAction::Get { id } => run_get(id, client).await,
        DocAction::Update(update_args) => run_update(update_args, client).await,
        DocAction::Delete { id, confirm } => run_delete(id, *confirm, client).await,
        DocAction::Link { doc_id, epic_id } => run_link(doc_id, *epic_id, client).await,
        DocAction::Unlink { doc_id, epic_id } => run_unlink(doc_id, *epic_id, client).await,
        DocAction::Epics { id } => run_epics(id, client).await,
    }
}

async fn run_list(client: &api::Client) -> Result<(), Box<dyn Error>> {
    let docs = client
        .list_docs()
        .send()
        .await
        .map_err(|e| format!("Failed to list documents: {e}"))?;

    for doc in docs.iter() {
        let title = doc.title.as_deref().unwrap_or("(untitled)");
        println!("{} - {}", doc.id, title);
    }

    if docs.is_empty() {
        println!("No documents found");
    }

    Ok(())
}

async fn run_create(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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

async fn run_get(id: &str, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let doc = client
        .get_doc()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get document: {e}"))?;

    let title = doc.title.as_deref().unwrap_or("(untitled)");
    println!("{} - {}", doc.id, title);
    println!("  Archived:  {}", doc.archived);
    println!("  Created:   {}", doc.created_at);
    println!("  Updated:   {}", doc.updated_at);
    if let Some(content) = &doc.content_markdown {
        println!();
        println!("{content}");
    }

    Ok(())
}

async fn run_update(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
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

async fn run_delete(id: &str, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a document is irreversible. Pass --confirm to proceed.".into());
    }

    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let doc = client
        .get_doc()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get document: {e}"))?;

    let title = doc.title.as_deref().unwrap_or("(untitled)").to_string();

    client
        .delete_doc()
        .doc_public_id(doc_id)
        .body_map(|b| b)
        .send()
        .await
        .map_err(|e| format!("Failed to delete document: {e}"))?;

    println!("Deleted document {id} - {title}");
    Ok(())
}

async fn run_link(doc_id: &str, epic_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_uuid: uuid::Uuid = doc_id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    client
        .link_document_to_epic()
        .doc_public_id(doc_uuid)
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to link document to epic: {e}"))?;

    println!("Linked document {doc_id} to epic {epic_id}");
    Ok(())
}

async fn run_unlink(
    doc_id: &str,
    epic_id: i64,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let doc_uuid: uuid::Uuid = doc_id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    client
        .unlink_document_from_epic()
        .doc_public_id(doc_uuid)
        .epic_public_id(epic_id)
        .send()
        .await
        .map_err(|e| format!("Failed to unlink document from epic: {e}"))?;

    println!("Unlinked document {doc_id} from epic {epic_id}");
    Ok(())
}

async fn run_epics(id: &str, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let doc_id: uuid::Uuid = id
        .parse()
        .map_err(|e| format!("Invalid document UUID: {e}"))?;

    let epics = client
        .list_document_epics()
        .doc_public_id(doc_id)
        .send()
        .await
        .map_err(|e| format!("Failed to list document epics: {e}"))?;

    for epic in epics.iter() {
        println!("{} - {}", epic.id, epic.name);
    }

    if epics.is_empty() {
        println!("No epics linked to this document");
    }

    Ok(())
}
