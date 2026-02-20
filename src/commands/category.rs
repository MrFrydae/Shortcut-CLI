use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct CategoryArgs {
    #[command(subcommand)]
    pub action: CategoryAction,
}

#[derive(Subcommand)]
pub enum CategoryAction {
    /// List all categories
    List {
        /// Include archived categories
        #[arg(long)]
        archived: bool,
    },
    /// Create a new category
    Create(Box<CreateArgs>),
    /// Get a category by ID
    Get {
        /// The ID of the category
        #[arg(long)]
        id: i64,
    },
    /// Update a category
    Update(Box<UpdateArgs>),
    /// Delete a category
    Delete {
        /// The ID of the category to delete
        #[arg(long)]
        id: i64,
        /// Confirm the irreversible deletion
        #[arg(long)]
        confirm: bool,
    },
    /// List milestones associated with a category
    Milestones {
        /// The ID of the category
        #[arg(long)]
        id: i64,
    },
    /// List objectives associated with a category
    Objectives {
        /// The ID of the category
        #[arg(long)]
        id: i64,
        /// Include objective descriptions in output
        #[arg(long, visible_alias = "descriptions")]
        desc: bool,
    },
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CreateArgs {
    /// The category name
    #[arg(long)]
    pub name: String,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// The category type (e.g. "milestone")
    #[arg(long, name = "type")]
    pub category_type: Option<String>,

    /// An external unique ID for import tracking
    #[arg(long)]
    pub external_id: Option<String>,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct UpdateArgs {
    /// The ID of the category to update
    #[arg(long)]
    pub id: i64,

    /// The category name
    #[arg(long)]
    pub name: Option<String>,

    /// The hex color (e.g. "#ff0000")
    #[arg(long)]
    pub color: Option<String>,

    /// Whether the category is archived
    #[arg(long)]
    pub archived: Option<bool>,
}

pub async fn run(args: &CategoryArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        CategoryAction::List { archived } => run_list(*archived, client).await,
        CategoryAction::Create(create_args) => run_create(create_args, client).await,
        CategoryAction::Get { id } => run_get(*id, client).await,
        CategoryAction::Update(update_args) => run_update(update_args, client).await,
        CategoryAction::Delete { id, confirm } => run_delete(*id, *confirm, client).await,
        CategoryAction::Milestones { id } => run_milestones(*id, client).await,
        CategoryAction::Objectives { id, desc } => run_objectives(*id, *desc, client).await,
    }
}

async fn run_list(include_archived: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let categories = client
        .list_categories()
        .send()
        .await
        .map_err(|e| format!("Failed to list categories: {e}"))?;

    for cat in categories.iter() {
        if !include_archived && cat.archived {
            continue;
        }
        let color = cat.color.as_deref().unwrap_or("none");
        println!("{} - {} ({})", cat.id, cat.name, color);
    }
    Ok(())
}

async fn run_create(args: &CreateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .parse::<api::types::CreateCategoryName>()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let external_id = args
        .external_id
        .as_ref()
        .map(|id| id.parse::<api::types::CreateCategoryExternalId>())
        .transpose()
        .map_err(|e| format!("Invalid external ID: {e}"))?;

    let category = client
        .create_category()
        .body_map(|mut b| {
            b = b.name(name);
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(t) = &args.category_type {
                b = b.type_(Some(serde_json::Value::String(t.clone())));
            }
            if let Some(ext_id) = external_id {
                b = b.external_id(Some(ext_id));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create category: {e}"))?;

    let color = category.color.as_deref().unwrap_or("none");
    println!(
        "Created category {} - {} ({})",
        category.id, category.name, color
    );
    Ok(())
}

async fn run_get(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let category = client
        .get_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get category: {e}"))?;

    println!("{} - {}", category.id, category.name);
    println!(
        "  Color:       {}",
        category.color.as_deref().unwrap_or("none")
    );
    println!("  Type:        {}", category.type_);
    println!("  Archived:    {}", category.archived);

    // Show associated milestones
    let milestones = client
        .list_category_milestones()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category milestones: {e}"))?;

    if !milestones.is_empty() {
        println!("  Milestones:");
        for ms in milestones.iter() {
            println!("    {} - {} ({})", ms.id, ms.name, ms.state);
        }
    }

    // Show associated objectives
    let objectives = client
        .list_category_objectives()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category objectives: {e}"))?;

    if !objectives.is_empty() {
        println!("  Objectives:");
        for obj in objectives.iter() {
            println!("    {} - {} ({})", obj.id, obj.name, obj.state);
        }
    }

    Ok(())
}

async fn run_update(args: &UpdateArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let name = args
        .name
        .as_ref()
        .map(|n| n.parse::<api::types::UpdateCategoryName>())
        .transpose()
        .map_err(|e| format!("Invalid name: {e}"))?;

    let category = client
        .update_category()
        .category_public_id(args.id)
        .body_map(|mut b| {
            if let Some(name) = name {
                b = b.name(Some(name));
            }
            if let Some(color) = &args.color {
                b = b.color(Some(color.clone()));
            }
            if let Some(archived) = args.archived {
                b = b.archived(Some(archived));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update category: {e}"))?;

    println!("Updated category {} - {}", category.id, category.name);
    Ok(())
}

async fn run_delete(id: i64, confirm: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if !confirm {
        return Err("Deleting a category is irreversible. Pass --confirm to proceed.".into());
    }

    let category = client
        .get_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to get category: {e}"))?;

    let name = category.name.clone();

    client
        .delete_category()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete category: {e}"))?;

    println!("Deleted category {id} - {name}");
    Ok(())
}

async fn run_milestones(id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let milestones = client
        .list_category_milestones()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category milestones: {e}"))?;

    for ms in milestones.iter() {
        println!("{} - {} ({})", ms.id, ms.name, ms.state);
    }

    if milestones.is_empty() {
        println!("No milestones for this category");
    }

    Ok(())
}

async fn run_objectives(id: i64, desc: bool, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let objectives = client
        .list_category_objectives()
        .category_public_id(id)
        .send()
        .await
        .map_err(|e| format!("Failed to list category objectives: {e}"))?;

    for obj in objectives.iter() {
        println!("{} - {} ({})", obj.id, obj.name, obj.state);
        if desc {
            println!("  {}", obj.description);
        }
    }

    if objectives.is_empty() {
        println!("No objectives for this category");
    }

    Ok(())
}
