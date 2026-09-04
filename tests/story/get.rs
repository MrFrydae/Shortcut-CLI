use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    branch_json, custom_field_enum_value_json, custom_field_json, full_story_json,
    full_story_json_with_branches_and_prs, label_json, pull_request_json, story_custom_field_json,
    typed_story_link_json,
};
use crate::{UUID_ALICE, UUID_FIELD_1, UUID_VAL_A, UUID_VAL_B};
use shortcut_cli::output::{ColorMode, OutputConfig, OutputMode};
use shortcut_cli::{api, commands::story};

const UUID_GROUP: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";

/// A full story exercising every field the machine-readable output must expose.
fn rich_story_json(id: i64) -> serde_json::Value {
    let mut body = full_story_json_with_branches_and_prs(
        id,
        "Rich Story",
        "Everything set",
        vec![branch_json(1, "feature/sc-7-rich", 42)],
        vec![pull_request_json(100, 456, "Rich PR", true, false, false)],
    );
    body["group_id"] = serde_json::json!(UUID_GROUP);
    body["iteration_id"] = serde_json::json!(88);
    body["epic_id"] = serde_json::json!(9);
    body["owner_ids"] = serde_json::json!([UUID_ALICE]);
    body["label_ids"] = serde_json::json!([1]);
    body["labels"] = serde_json::json!([label_json(1, "bug")]);
    body["custom_fields"] =
        serde_json::json!([story_custom_field_json(UUID_FIELD_1, UUID_VAL_A, "High")]);
    body["story_links"] =
        serde_json::json!([typed_story_link_json(5, id, 12, "blocks", "subject")]);
    body
}

/// Mount the custom-fields endpoint used to resolve `field_name`.
async fn mount_custom_fields(server: &MockServer, expected_calls: u64) {
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![
            custom_field_enum_value_json(UUID_VAL_A, "High", 0, true),
            custom_field_enum_value_json(UUID_VAL_B, "Low", 1, true),
        ]
    )]);
    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(expected_calls)
        .mount(server)
        .await;
}

async fn run_get(mode: OutputMode, server: &MockServer, tmp: &std::path::Path) -> String {
    let (out, buf) = OutputConfig::with_buffer(mode, ColorMode::Never);
    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 7 },
    };
    let result = story::run(&args, &client, tmp.to_path_buf(), &out).await;
    assert!(result.is_ok(), "{result:?}");
    String::from_utf8(buf.lock().unwrap().clone()).unwrap()
}

#[tokio::test]
async fn get_story_json_emits_full_story() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rich_story_json(7)))
        .expect(1)
        .mount(&server)
        .await;
    mount_custom_fields(&server, 1).await;

    let output = run_get(OutputMode::Json, &server, tmp.path()).await;
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();

    // Previously omitted fields are now present.
    assert_eq!(json["group_id"], UUID_GROUP);
    assert_eq!(json["iteration_id"], 88);
    assert_eq!(json["app_url"], "https://app.shortcut.com/test/story/7");
    assert_eq!(json["created_at"], "2024-01-01T00:00:00Z");
    assert_eq!(json["updated_at"], "2024-01-01T00:00:00Z");
    assert_eq!(json["label_ids"], serde_json::json!([1]));
    assert_eq!(json["story_links"][0]["id"], 5);
    assert_eq!(json["story_links"][0]["verb"], "blocks");
    assert_eq!(json["story_links"][0]["object_id"], 12);
    assert_eq!(json["story_links"][0]["type"], "subject");

    // Custom fields carry ids plus resolved names.
    let cf = &json["custom_fields"][0];
    assert_eq!(cf["field_id"], UUID_FIELD_1);
    assert_eq!(cf["value_id"], UUID_VAL_A);
    assert_eq!(cf["value"], "High");
    assert_eq!(cf["field_name"], "Priority");

    // Existing shapes are unchanged.
    assert_eq!(json["id"], 7);
    assert_eq!(json["name"], "Rich Story");
    assert_eq!(json["story_type"], "feature");
    assert_eq!(json["epic_id"], 9);
    assert_eq!(json["owner_ids"], serde_json::json!([UUID_ALICE]));
    assert_eq!(json["labels"], serde_json::json!(["bug"]));
    assert_eq!(json["label_details"][0]["id"], 1);
    assert_eq!(json["label_details"][0]["name"], "bug");
    assert_eq!(json["branches"][0]["name"], "feature/sc-7-rich");
    assert_eq!(json["branches"][0]["repository_id"], 42);
    let pr = &json["pull_requests"][0];
    assert_eq!(pr["number"], 456);
    assert_eq!(pr["title"], "Rich PR");
    assert_eq!(pr["repository_id"], 42);
    assert_eq!(pr["status"], "merged");
    assert_eq!(pr["merged"], true);
}

#[tokio::test]
async fn get_story_toon_matches_json_object() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rich_story_json(7)))
        .expect(2)
        .mount(&server)
        .await;
    // Second run is served from the custom field cache written by the first.
    mount_custom_fields(&server, 1).await;

    let json_output = run_get(OutputMode::Json, &server, tmp.path()).await;
    let toon_output = run_get(OutputMode::Toon, &server, tmp.path()).await;

    let json: serde_json::Value = serde_json::from_str(json_output.trim()).unwrap();
    assert_eq!(toon_output, format!("{}\n", toon::encode(&json, None)));
    assert!(toon_output.contains("field_name"));
    assert!(toon_output.contains("Priority"));
    assert!(toon_output.contains(UUID_GROUP));
}

#[tokio::test]
async fn get_story_json_without_custom_fields_skips_lookup() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(full_story_json(7, "Plain", "")))
        .expect(1)
        .mount(&server)
        .await;
    // No /custom-fields mock: a lookup would 404 and fail the command.

    let output = run_get(OutputMode::Json, &server, tmp.path()).await;
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(json["custom_fields"], serde_json::json!([]));
    assert_eq!(json["labels"], serde_json::json!([]));
    assert!(json["group_id"].is_null());

    // Keys the generated types would omit when unset are still present.
    let obj = json.as_object().unwrap();
    for key in ["parent_story_id", "cycle_time", "lead_time", "synced_item"] {
        assert!(obj.contains_key(key), "missing key {key}");
        assert!(json[key].is_null(), "{key} should be null");
    }
    assert_eq!(json["sub_task_story_ids"], serde_json::json!([]));
}

#[tokio::test]
async fn get_story_prints_details() {
    let out = crate::support::make_output();
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json(99, "Important Story", "Some description");

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 99 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_story_shows_branches_and_prs() {
    let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = full_story_json_with_branches_and_prs(
        99,
        "Login Fix",
        "Fix the login",
        vec![branch_json(1, "feature/sc-99-login-fix", 42)],
        vec![pull_request_json(
            100,
            456,
            "Fix login bug",
            true,
            false,
            false,
        )],
    );

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 99 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf(), &out).await;
    assert!(result.is_ok());

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(output.contains("Branches:"));
    assert!(output.contains("feature/sc-99-login-fix"));
    assert!(output.contains("repo 42"));
    assert!(output.contains("Pull Requests:"));
    assert!(output.contains("#456"));
    assert!(output.contains("Fix login bug"));
    assert!(output.contains("merged"));
}
