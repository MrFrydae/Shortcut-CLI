mod support;

use sc::{api, commands::doc};
use support::{doc_json, doc_slim_json, epic_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DOC_UUID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
const DOC_UUID2: &str = "11111111-2222-3333-4444-555555555555";

// --- List tests ---

#[tokio::test]
async fn list_docs() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        doc_slim_json(DOC_UUID, Some("Design Spec")),
        doc_slim_json(DOC_UUID2, Some("RFC: New API")),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::List,
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_docs_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::List,
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Create tests ---

#[tokio::test]
async fn create_doc_minimal() {
    let server = MockServer::start().await;

    let body = doc_slim_json(DOC_UUID, Some("My Document"));

    Mock::given(method("POST"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Create(Box::new(doc::CreateArgs {
            title: "My Document".to_string(),
            content: Some("Hello world".to_string()),
            content_file: None,
            content_format: None,
        })),
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_doc_with_format() {
    let server = MockServer::start().await;

    let body = doc_slim_json(DOC_UUID, Some("Markdown Doc"));

    Mock::given(method("POST"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Create(Box::new(doc::CreateArgs {
            title: "Markdown Doc".to_string(),
            content: Some("# Title\nBody text".to_string()),
            content_file: None,
            content_format: Some("markdown".to_string()),
        })),
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_doc_with_content_file() {
    let server = MockServer::start().await;

    let body = doc_slim_json(DOC_UUID, Some("File Doc"));

    Mock::given(method("POST"))
        .and(path("/api/v3/documents"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "# Content from file\nSome body").unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Create(Box::new(doc::CreateArgs {
            title: "File Doc".to_string(),
            content: None,
            content_file: Some(tmp.path().to_path_buf()),
            content_format: Some("markdown".to_string()),
        })),
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Get tests ---

#[tokio::test]
async fn get_doc() {
    let server = MockServer::start().await;

    let body = doc_json(DOC_UUID, Some("My Document"), Some("# Hello\nWorld"));

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Get {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_doc_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Get {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_err());
}

// --- Update tests ---

#[tokio::test]
async fn update_doc_title() {
    let server = MockServer::start().await;

    let body = doc_json(DOC_UUID, Some("New Title"), Some("content"));

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Update(Box::new(doc::UpdateArgs {
            id: DOC_UUID.to_string(),
            title: Some("New Title".to_string()),
            content: None,
            content_file: None,
            content_format: None,
        })),
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_doc_content() {
    let server = MockServer::start().await;

    let body = doc_json(DOC_UUID, Some("My Doc"), Some("updated content"));

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Update(Box::new(doc::UpdateArgs {
            id: DOC_UUID.to_string(),
            title: None,
            content: Some("updated content".to_string()),
            content_file: None,
            content_format: None,
        })),
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Delete tests ---

#[tokio::test]
async fn delete_doc_with_confirm() {
    let server = MockServer::start().await;

    let get_body = doc_json(DOC_UUID, Some("To Delete"), Some("content"));

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Delete {
            id: DOC_UUID.to_string(),
            confirm: true,
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_doc_without_confirm_errors() {
    let server = MockServer::start().await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Delete {
            id: DOC_UUID.to_string(),
            confirm: false,
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("--confirm"));
}

// --- Link tests ---

#[tokio::test]
async fn link_doc_to_epic() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics/42")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Link {
            doc_id: DOC_UUID.to_string(),
            epic_id: 42,
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Unlink tests ---

#[tokio::test]
async fn unlink_doc_from_epic() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics/42")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Unlink {
            doc_id: DOC_UUID.to_string(),
            epic_id: 42,
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

// --- Document epics tests ---

#[tokio::test]
async fn list_doc_epics() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        epic_json(1, "Epic One", None),
        epic_json(2, "Epic Two", None),
    ]);

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Epics {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_doc_epics_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v3/documents/{DOC_UUID}/epics")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = doc::DocArgs {
        action: doc::DocAction::Epics {
            id: DOC_UUID.to_string(),
        },
    };
    let result = doc::run(&args, &client).await;
    assert!(result.is_ok());
}
