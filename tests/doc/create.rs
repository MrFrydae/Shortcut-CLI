use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::DOC_UUID;
use crate::support::doc_slim_json;
use sc::{api, commands::doc};

#[tokio::test]
async fn create_doc_minimal() {
    let out = crate::support::make_output();
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
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_doc_with_format() {
    let out = crate::support::make_output();
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
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_doc_with_content_file() {
    let out = crate::support::make_output();
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
    let result = doc::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
