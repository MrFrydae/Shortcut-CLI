use std::error::Error;

use clap::{Args, Subcommand};

use crate::api;

#[derive(Args)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub action: TaskAction,
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add one or more tasks to a story
    Add(AddArgs),
    /// List all tasks on a story
    List {
        /// The story ID
        #[arg(long)]
        story_id: i64,
    },
    /// Get a single task
    Get {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
    /// Mark a task as complete
    Check {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
    /// Mark a task as incomplete
    Uncheck {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
    /// Update a task's description
    Update(UpdateTaskArgs),
    /// Delete a task
    Delete {
        /// The story ID
        #[arg(long)]
        story_id: i64,
        /// The task ID
        #[arg(long)]
        id: i64,
    },
}

#[derive(Args)]
pub struct AddArgs {
    /// The story ID
    #[arg(long)]
    pub story_id: i64,
    /// Task description(s) — repeat for multiple tasks
    #[arg(long)]
    pub description: Vec<String>,
}

#[derive(Args)]
pub struct UpdateTaskArgs {
    /// The story ID
    #[arg(long)]
    pub story_id: i64,
    /// The task ID
    #[arg(long)]
    pub id: i64,
    /// New description for the task
    #[arg(long)]
    pub description: Option<String>,
    /// Mark the task as complete or incomplete
    #[arg(long)]
    pub complete: Option<bool>,
}

pub async fn run(args: &TaskArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    match &args.action {
        TaskAction::Add(add_args) => run_add(add_args, client).await,
        TaskAction::List { story_id } => run_list(*story_id, client).await,
        TaskAction::Get { story_id, id } => run_get(*story_id, *id, client).await,
        TaskAction::Check { story_id, id } => run_set_complete(*story_id, *id, true, client).await,
        TaskAction::Uncheck { story_id, id } => {
            run_set_complete(*story_id, *id, false, client).await
        }
        TaskAction::Update(update_args) => run_update(update_args, client).await,
        TaskAction::Delete { story_id, id } => run_delete(*story_id, *id, client).await,
    }
}

async fn run_add(args: &AddArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    if args.description.is_empty() {
        return Err("At least one --description is required".into());
    }

    let descriptions: Vec<api::types::CreateTaskDescription> = args
        .description
        .iter()
        .map(|d| {
            d.parse::<api::types::CreateTaskDescription>()
                .map_err(|e| format!("Invalid description '{}': {e}", d))
        })
        .collect::<Result<_, _>>()?;

    let mut errors = Vec::new();
    for desc in descriptions {
        match client
            .create_task()
            .story_public_id(args.story_id)
            .body_map(|b| b.description(desc.clone()))
            .send()
            .await
        {
            Ok(task) => println!("Created task {} - {}", task.id, task.description),
            Err(e) => errors.push(format!("Failed to create task '{}': {e}", &*desc)),
        }
    }

    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(())
}

async fn run_list(story_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let story = client
        .get_story()
        .story_public_id(story_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get story: {e}"))?;

    if story.tasks.is_empty() {
        println!("No tasks on story {story_id}");
        return Ok(());
    }

    for task in &story.tasks {
        let check = if task.complete { "x" } else { " " };
        println!("[{check}] {} - {}", task.id, task.description);
    }
    Ok(())
}

async fn run_get(story_id: i64, task_id: i64, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let task = client
        .get_task()
        .story_public_id(story_id)
        .task_public_id(task_id)
        .send()
        .await
        .map_err(|e| format!("Failed to get task: {e}"))?;

    let check = if task.complete { "x" } else { " " };
    println!("[{check}] {} - {}", task.id, task.description);
    Ok(())
}

async fn run_set_complete(
    story_id: i64,
    task_id: i64,
    complete: bool,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    let task = client
        .update_task()
        .story_public_id(story_id)
        .task_public_id(task_id)
        .body_map(|b| b.complete(Some(complete)))
        .send()
        .await
        .map_err(|e| format!("Failed to update task: {e}"))?;

    let status = if task.complete {
        "complete"
    } else {
        "incomplete"
    };
    println!("Task {} marked {status}", task.id);
    Ok(())
}

async fn run_update(args: &UpdateTaskArgs, client: &api::Client) -> Result<(), Box<dyn Error>> {
    let description = args
        .description
        .as_ref()
        .map(|d| d.parse::<api::types::UpdateTaskDescription>())
        .transpose()
        .map_err(|e| format!("Invalid description: {e}"))?;

    let task = client
        .update_task()
        .story_public_id(args.story_id)
        .task_public_id(args.id)
        .body_map(|mut b| {
            if let Some(desc) = description {
                b = b.description(Some(desc));
            }
            if let Some(complete) = args.complete {
                b = b.complete(Some(complete));
            }
            b
        })
        .send()
        .await
        .map_err(|e| format!("Failed to update task: {e}"))?;

    println!("Updated task {} - {}", task.id, task.description);
    Ok(())
}

async fn run_delete(
    story_id: i64,
    task_id: i64,
    client: &api::Client,
) -> Result<(), Box<dyn Error>> {
    client
        .delete_task()
        .story_public_id(story_id)
        .task_public_id(task_id)
        .send()
        .await
        .map_err(|e| format!("Failed to delete task: {e}"))?;

    println!("Deleted task {task_id} from story {story_id}");
    Ok(())
}
