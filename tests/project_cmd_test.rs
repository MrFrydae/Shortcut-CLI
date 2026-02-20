mod support;

use sc::{api, commands::project};
use support::{project_json, story_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --- List tests ---

#[tokio::test]
async fn list_projects() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        project_json(1, "Backend", Some("Backend services")),
        project_json(2, "Frontend", Some("Frontend apps")),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::List { archived: false },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_projects_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::List { archived: false },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_projects_filters_archived() {
    let server = MockServer::start().await;

    let mut archived_proj = project_json(2, "Old Project", None);
    archived_proj["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        project_json(1, "Active Project", Some("Still active")),
        archived_proj,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::List { archived: false },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Create tests ---

#[tokio::test]
async fn create_project_minimal() {
    let server = MockServer::start().await;

    let body = project_json(42, "New Project", None);

    Mock::given(method("POST"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Create(Box::new(project::CreateArgs {
            name: "New Project".to_string(),
            team_id: 1,
            description: None,
            color: None,
            abbreviation: None,
            iteration_length: None,
            external_id: None,
        })),
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_project_with_all_fields() {
    let server = MockServer::start().await;

    let body = project_json(43, "Full Project", Some("A fully specified project"));

    Mock::given(method("POST"))
        .and(path("/api/v3/projects"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Create(Box::new(project::CreateArgs {
            name: "Full Project".to_string(),
            team_id: 1,
            description: Some("A fully specified project".to_string()),
            color: Some("#ff0000".to_string()),
            abbreviation: Some("FP".to_string()),
            iteration_length: Some(3),
            external_id: Some("ext-123".to_string()),
        })),
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Get tests ---

#[tokio::test]
async fn get_project() {
    let server = MockServer::start().await;

    let body = project_json(42, "My Project", Some("A great project"));

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Get { id: 42 },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_project_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Get { id: 999 },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_err());
}

// --- Update test ---

#[tokio::test]
async fn update_project() {
    let server = MockServer::start().await;

    let body = project_json(42, "Updated Project", Some("Updated description"));

    Mock::given(method("PUT"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Update(Box::new(project::UpdateArgs {
            id: 42,
            name: Some("Updated Project".to_string()),
            description: None,
            color: None,
            abbreviation: None,
            archived: None,
            team_id: None,
            days_to_thermometer: None,
            show_thermometer: None,
        })),
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_project_with_confirm() {
    let server = MockServer::start().await;

    let body = project_json(42, "To Delete", None);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/projects/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_project_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

// --- Stories listing tests ---

#[tokio::test]
async fn list_project_stories() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        story_json(1, "Story A", None),
        story_json(2, "Story B", None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/42/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Stories {
            id: 42,
            desc: false,
        },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_project_stories_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/projects/42/stories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = project::ProjectArgs {
        action: project::ProjectAction::Stories {
            id: 42,
            desc: false,
        },
    };
    let result = project::run(&args, &client).await;
    assert!(result.is_ok());
}
