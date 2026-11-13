use std::path::{Path, PathBuf};

use wiremock::matchers::{body_partial_json, body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::{api, commands::file, commands::story};

use crate::support::{full_story_json, make_dry_run_output, make_output, uploaded_file_json};

const MEDIA: &str = "https://media.app.shortcut.com/api/attachments/files/clubhouse-assets";

fn write_files(dir: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|n| {
            let p = dir.join(n);
            std::fs::write(&p, format!("contents of {n}")).unwrap();
            p
        })
        .collect()
}

fn upload_args(paths: Vec<PathBuf>, story_id: Option<i64>) -> file::FileArgs {
    file::FileArgs {
        action: file::FileAction::Upload(Box::new(file::UploadArgs {
            paths,
            story_id,
            markdown: false,
            embed: false,
        })),
    }
}

fn with_flags(mut args: file::FileArgs, markdown: bool, embed: bool) -> file::FileArgs {
    if let file::FileAction::Upload(u) = &mut args.action {
        u.markdown = markdown;
        u.embed = embed;
    }
    args
}

fn output_of(buf: &shortcut_cli::output::SharedBuffer) -> String {
    String::from_utf8(buf.lock().unwrap().clone()).unwrap()
}

#[tokio::test]
async fn upload_sends_multipart_with_story_id() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .and(header("Shortcut-Token", "test-token"))
        .and(body_string_contains("name=\"story_id\""))
        .and(body_string_contains("6395"))
        .and(body_string_contains("name=\"file0\"; filename=\"a.png\""))
        .and(body_string_contains("image/png"))
        .and(body_string_contains("contents of a.png"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![6395])
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(&upload_args(paths, Some(6395)), &client, &out)
        .await
        .unwrap();

    let output = output_of(&buf);
    assert!(output.contains("Uploaded file #1 a.png (2.0 KB)"));
    assert!(output.contains(&format!("{MEDIA}/1/a.png")));
    assert!(output.contains("Attached to story 6395"));
}

#[tokio::test]
async fn upload_without_story_id_omits_field() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["notes.txt"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "notes.txt", "text/plain", vec![])
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(&upload_args(paths, None), &client, &out)
        .await
        .unwrap();

    let reqs = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&reqs[0].body);
    assert!(!body.contains("story_id"));
}

#[tokio::test]
async fn upload_more_than_four_files_batches_requests() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Quiet, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "b.png", "c.png", "d.png", "e.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .and(body_string_contains("filename=\"e.png\""))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(5, "e.png", "image/png", vec![1])
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![1]),
            uploaded_file_json(2, "b.png", "image/png", vec![1]),
            uploaded_file_json(3, "c.png", "image/png", vec![1]),
            uploaded_file_json(4, "d.png", "image/png", vec![1]),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(&upload_args(paths, Some(1)), &client, &out)
        .await
        .unwrap();

    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 2);
    let first = String::from_utf8_lossy(&reqs[0].body);
    assert!(first.contains("name=\"file3\"; filename=\"d.png\""));
    assert!(!first.contains("file4"));
    let second = String::from_utf8_lossy(&reqs[1].body);
    assert!(second.contains("name=\"file0\"; filename=\"e.png\""));
    assert!(second.contains("name=\"story_id\""));

    assert_eq!(output_of(&buf), "1\n2\n3\n4\n5\n");
}

#[tokio::test]
async fn upload_partial_failure_reports_already_uploaded_ids() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "b.png", "c.png", "d.png", "e.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(11, "a.png", "image/png", vec![]),
            uploaded_file_json(12, "b.png", "image/png", vec![]),
            uploaded_file_json(13, "c.png", "image/png", vec![]),
            uploaded_file_json(14, "d.png", "image/png", vec![]),
        ])))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(
            ResponseTemplate::new(500).set_body_json(serde_json::json!({"message": "boom"})),
        )
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let err = file::run(&upload_args(paths, None), &client, &out)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("Failed to upload files: 500 Internal Server Error: boom"));
    assert!(err.contains("11, 12, 13, 14"));
}

#[tokio::test]
async fn upload_api_error_is_formatted() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(
            ResponseTemplate::new(422)
                .set_body_json(serde_json::json!({"message": "Story does not exist"})),
        )
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let err = file::run(&upload_args(paths, Some(999)), &client, &out)
        .await
        .unwrap_err()
        .to_string();
    assert_eq!(
        err,
        "Failed to upload files: 422 Unprocessable Entity: Story does not exist"
    );
}

#[tokio::test]
async fn upload_missing_file_fails_before_any_request() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let mut paths = write_files(tmp.path(), &["a.png"]);
    paths.push(tmp.path().join("missing.png"));

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let err = file::run(&upload_args(paths, None), &client, &out)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("missing.png"));
}

#[tokio::test]
async fn upload_dry_run_validates_and_sends_nothing() {
    let (out, buf) = make_dry_run_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "b.png", "c.png", "d.png", "e.pdf"]);

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(
        &with_flags(upload_args(paths, Some(7)), false, true),
        &client,
        &out,
    )
    .await
    .unwrap();

    let output = output_of(&buf);
    assert_eq!(
        output
            .matches("[dry-run] POST /api/v3/files (multipart/form-data)")
            .count(),
        2
    );
    assert!(output.contains("\"content_type\": \"application/pdf\""));
    assert!(output.contains("\"field\": \"file3\""));
    assert!(output.contains("\"story_id\": 7"));
    assert!(output.contains("[dry-run] GET /api/v3/stories/7"));
    assert!(output.contains("[dry-run] PUT /api/v3/stories/7"));
}

#[tokio::test]
async fn upload_markdown_output() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "spec.pdf"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![]),
            uploaded_file_json(2, "spec.pdf", "application/pdf", vec![]),
        ])))
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(
        &with_flags(upload_args(paths, None), true, false),
        &client,
        &out,
    )
    .await
    .unwrap();

    assert_eq!(
        output_of(&buf),
        format!("![a.png]({MEDIA}/1/a.png)\n[spec.pdf]({MEDIA}/2/spec.pdf)\n")
    );
}

#[tokio::test]
async fn upload_json_output_includes_markdown_field() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Json, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![]),
        ])))
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(&upload_args(paths, None), &client, &out)
        .await
        .unwrap();

    let json: serde_json::Value = serde_json::from_str(&output_of(&buf)).unwrap();
    assert_eq!(json[0]["id"], 1);
    assert_eq!(json[0]["markdown"], format!("![a.png]({MEDIA}/1/a.png)"));
}

#[tokio::test]
async fn upload_format_template_output() {
    let (out, buf) = OutputConfig::with_buffer(
        OutputMode::Format("{id}\t{markdown}".to_string()),
        ColorMode::Never,
    );
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![]),
        ])))
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(&upload_args(paths, None), &client, &out)
        .await
        .unwrap();
    assert_eq!(output_of(&buf), format!("1\t![a.png]({MEDIA}/1/a.png)\n"));
}

#[tokio::test]
async fn upload_embed_appends_markdown_to_story_description() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "b.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(1, "a.png", "image/png", vec![42]),
            uploaded_file_json(2, "b.png", "image/png", vec![42]),
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(full_story_json(
            42,
            "Story",
            "Existing\n",
        )))
        .expect(1)
        .mount(&server)
        .await;
    let expected = format!("Existing\n\n![a.png]({MEDIA}/1/a.png)\n\n![b.png]({MEDIA}/2/b.png)");
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .and(body_partial_json(
            serde_json::json!({ "description": expected }),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(full_story_json(42, "Story", &expected)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    file::run(
        &with_flags(upload_args(paths, Some(42)), false, true),
        &client,
        &out,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn upload_embed_failure_reports_uploaded_files() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "b.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(21, "a.png", "image/png", vec![42]),
            uploaded_file_json(22, "b.png", "image/png", vec![42]),
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(full_story_json(42, "Story", "Existing")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(
            ResponseTemplate::new(422)
                .set_body_json(serde_json::json!({"message": "Description is invalid"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let err = file::run(
        &with_flags(upload_args(paths, Some(42)), false, true),
        &client,
        &out,
    )
    .await
    .unwrap_err()
    .to_string();

    assert!(err.contains("failed to update story 42 description"));
    assert!(err.contains("422 Unprocessable Entity: Description is invalid"));
    assert!(err.contains("(file IDs): 21, 22"));
    assert!(err.contains(&format!("![a.png]({MEDIA}/21/a.png)")));
    assert!(err.contains(&format!("![b.png]({MEDIA}/22/b.png)")));
}

#[tokio::test]
async fn upload_partial_failure_includes_markdown() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png", "b.png", "c.png", "d.png", "e.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(31, "a.png", "image/png", vec![]),
            uploaded_file_json(32, "b.png", "image/png", vec![]),
            uploaded_file_json(33, "c.png", "image/png", vec![]),
            uploaded_file_json(34, "d.png", "image/png", vec![]),
        ])))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let err = file::run(&upload_args(paths, None), &client, &out)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains(&format!("![d.png]({MEDIA}/34/d.png)")));
}

#[tokio::test]
async fn upload_embed_requires_story_id() {
    let out = make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png"]);

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let err = file::run(
        &with_flags(upload_args(paths, None), false, true),
        &client,
        &out,
    )
    .await
    .unwrap_err()
    .to_string();
    assert!(err.contains("--story-id"));
}

#[tokio::test]
async fn story_file_upload_alias_attaches_to_story() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Quiet, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let paths = write_files(tmp.path(), &["a.png"]);

    Mock::given(method("POST"))
        .and(path("/api/v3/files"))
        .and(body_string_contains("name=\"story_id\""))
        .and(body_string_contains("123"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!([
            uploaded_file_json(9, "a.png", "image/png", vec![123]),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::File(story::file::StoryFileArgs {
            action: story::file::StoryFileAction::Upload {
                story_id: 123,
                paths,
                markdown: false,
                embed: false,
            },
        }),
    };
    story::run(&args, &client, cache.path().to_path_buf(), &out)
        .await
        .unwrap();
    assert_eq!(output_of(&buf), "9\n");
}
