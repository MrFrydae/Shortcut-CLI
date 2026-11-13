use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::{api, commands::file};

use crate::support::{make_dry_run_output, make_output, uploaded_file_json};

fn output_of(buf: &shortcut_cli::output::SharedBuffer) -> String {
    String::from_utf8(buf.lock().unwrap().clone()).unwrap()
}

#[tokio::test]
async fn list_files_table() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![10, 11]),
            uploaded_file_json(2, "b.pdf", "application/pdf", vec![]),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = file::FileArgs {
        action: file::FileAction::List,
    };
    file::run(&args, &client, &out).await.unwrap();
    let output = output_of(&buf);
    assert!(output.contains("a.png"));
    assert!(output.contains("10, 11"));
    assert!(output.contains("application/pdf"));
}

#[tokio::test]
async fn get_file_shows_markdown() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/files/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(uploaded_file_json(
            5,
            "shot.png",
            "image/png",
            vec![42],
        )))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = file::FileArgs {
        action: file::FileAction::Get { id: 5 },
    };
    file::run(&args, &client, &out).await.unwrap();
    let output = output_of(&buf);
    assert!(output.starts_with("5 - shot.png"));
    assert!(output.contains("Markdown:    ![shot.png](https://media.app.shortcut.com/"));
    assert!(output.contains("Stories:     42"));
}

#[tokio::test]
async fn update_file_name() {
    let out = make_output();
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v3/files/5"))
        .and(body_partial_json(
            serde_json::json!({"name": "renamed.png"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(uploaded_file_json(
            5,
            "renamed.png",
            "image/png",
            vec![],
        )))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = file::FileArgs {
        action: file::FileAction::Update(Box::new(file::UpdateArgs {
            id: 5,
            name: Some("renamed.png".to_string()),
            description: None,
        })),
    };
    file::run(&args, &client, &out).await.unwrap();
}

#[tokio::test]
async fn delete_file_with_confirm() {
    let out = make_output();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/files/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(uploaded_file_json(
            5,
            "a.png",
            "image/png",
            vec![],
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v3/files/5"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = file::FileArgs {
        action: file::FileAction::Delete {
            id: 5,
            confirm: true,
        },
    };
    file::run(&args, &client, &out).await.unwrap();
}

#[tokio::test]
async fn delete_file_without_confirm_errors() {
    let out = make_output();
    let server = MockServer::start().await;
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = file::FileArgs {
        action: file::FileAction::Delete {
            id: 5,
            confirm: false,
        },
    };
    let err = file::run(&args, &client, &out).await.unwrap_err();
    assert!(err.to_string().contains("--confirm"));
}

#[tokio::test]
async fn delete_file_dry_run() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = file::FileArgs {
        action: file::FileAction::Delete {
            id: 5,
            confirm: false,
        },
    };
    file::run(&args, &client, &out).await.unwrap();
    assert!(output_of(&buf).contains("[dry-run] DELETE /api/v3/files/5"));
}
