use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{full_story_json_with_tasks, task_json};
use sc::{api, commands::story, commands::story::task};

// --- Add tests ---

#[tokio::test]
async fn add_single_task() {
    let server = MockServer::start().await;

    let body = task_json(1, 123, "Write tests", false);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/123/tasks"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let add_args = task::AddArgs {
        story_id: 123,
        description: vec!["Write tests".to_string()],
    };
    let args = story::StoryArgs {
        action: story::StoryAction::Task(task::TaskArgs {
            action: task::TaskAction::Add(add_args),
        }),
    };
    let result = task::run(
        match &args.action {
            story::StoryAction::Task(t) => t,
            _ => unreachable!(),
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn add_multiple_tasks() {
    let server = MockServer::start().await;

    let body1 = task_json(1, 123, "Write tests", false);
    let body2 = task_json(2, 123, "Update docs", false);

    Mock::given(method("POST"))
        .and(path("/api/v3/stories/123/tasks"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body1))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second call returns different body
    Mock::given(method("POST"))
        .and(path("/api/v3/stories/123/tasks"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body2))
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let add_args = task::AddArgs {
        story_id: 123,
        description: vec!["Write tests".to_string(), "Update docs".to_string()],
    };
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Add(add_args),
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn add_empty_description_errors() {
    let server = MockServer::start().await;
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let add_args = task::AddArgs {
        story_id: 123,
        description: vec![],
    };
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Add(add_args),
        },
        &client,
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("--description"));
}

// --- List tests ---

#[tokio::test]
async fn list_tasks() {
    let server = MockServer::start().await;

    let tasks = vec![
        task_json(1, 123, "Write tests", true),
        task_json(2, 123, "Update docs", false),
    ];
    let body = full_story_json_with_tasks(123, "My Story", "desc", tasks);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::List { story_id: 123 },
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_tasks_empty() {
    let server = MockServer::start().await;

    let body = full_story_json_with_tasks(123, "My Story", "desc", vec![]);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::List { story_id: 123 },
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

// --- Get test ---

#[tokio::test]
async fn get_task() {
    let server = MockServer::start().await;

    let body = task_json(456, 123, "Write tests", false);

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/123/tasks/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Get {
                story_id: 123,
                id: 456,
            },
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

// --- Check/Uncheck tests ---

#[tokio::test]
async fn check_task() {
    let server = MockServer::start().await;

    let body = task_json(456, 123, "Write tests", true);

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/123/tasks/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Check {
                story_id: 123,
                id: 456,
            },
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn uncheck_task() {
    let server = MockServer::start().await;

    let body = task_json(456, 123, "Write tests", false);

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/123/tasks/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Uncheck {
                story_id: 123,
                id: 456,
            },
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

// --- Update test ---

#[tokio::test]
async fn update_task_description() {
    let server = MockServer::start().await;

    let body = task_json(456, 123, "New text", false);

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/123/tasks/456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let update_args = task::UpdateTaskArgs {
        story_id: 123,
        id: 456,
        description: Some("New text".to_string()),
        complete: None,
    };
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Update(update_args),
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}

// --- Delete test ---

#[tokio::test]
async fn delete_task() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/stories/123/tasks/456"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let result = task::run(
        &task::TaskArgs {
            action: task::TaskAction::Delete {
                story_id: 123,
                id: 456,
            },
        },
        &client,
    )
    .await;
    assert!(result.is_ok());
}
