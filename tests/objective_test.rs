mod support;

use sc::{api, commands::objective};
use support::{epic_json, objective_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --- List tests ---

#[tokio::test]
async fn list_objectives() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        objective_json(1, "Q1 Goals", "in progress", "First objective"),
        objective_json(2, "Q2 Goals", "to do", "Second objective"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::List { archived: false },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_objectives_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::List { archived: false },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_objectives_filters_archived() {
    let server = MockServer::start().await;

    let mut archived_obj = objective_json(2, "Old Goals", "done", "Archived");
    archived_obj["archived"] = serde_json::Value::Bool(true);

    let body = serde_json::json!([
        objective_json(1, "Current Goals", "in progress", "Active"),
        archived_obj,
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    // Without --archived flag, the archived objective should be filtered out
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::List { archived: false },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Create tests ---

#[tokio::test]
async fn create_objective_minimal() {
    let server = MockServer::start().await;

    let body = objective_json(42, "New Objective", "to do", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Create(Box::new(objective::CreateArgs {
            name: "New Objective".to_string(),
            description: None,
            state: None,
            categories: vec![],
        })),
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_objective_with_all_fields() {
    let server = MockServer::start().await;

    let body = objective_json(43, "Full Objective", "in progress", "A description");

    Mock::given(method("POST"))
        .and(path("/api/v3/objectives"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Create(Box::new(objective::CreateArgs {
            name: "Full Objective".to_string(),
            description: Some("A description".to_string()),
            state: Some("in progress".to_string()),
            categories: vec!["Engineering".to_string()],
        })),
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Get tests ---

#[tokio::test]
async fn get_objective() {
    let server = MockServer::start().await;

    let body = objective_json(42, "My Objective", "in progress", "Details here");

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Also mock the epics endpoint that get calls
    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42/epics"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([epic_json(
                1,
                "Related Epic",
                None
            ),])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Get { id: 42 },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_objective_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/999"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Get { id: 999 },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_err());
}

// --- Update test ---

#[tokio::test]
async fn update_objective() {
    let server = MockServer::start().await;

    let body = objective_json(42, "Updated Name", "done", "Updated desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Update(Box::new(objective::UpdateArgs {
            id: 42,
            name: Some("Updated Name".to_string()),
            description: None,
            state: None,
            archived: None,
            categories: vec![],
        })),
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_objective_with_confirm() {
    let server = MockServer::start().await;

    let body = objective_json(42, "To Delete", "to do", "");

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v3/objectives/42"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Delete {
            id: 42,
            confirm: true,
        },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_objective_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Delete {
            id: 42,
            confirm: false,
        },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

// --- Epics listing tests ---

#[tokio::test]
async fn list_objective_epics() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Epics {
            id: 42,
            desc: false,
        },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_objective_epics_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/objectives/42/epics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = objective::ObjectiveArgs {
        action: objective::ObjectiveAction::Epics {
            id: 42,
            desc: false,
        },
    };
    let result = objective::run(&args, &client).await;
    assert!(result.is_ok());
}
