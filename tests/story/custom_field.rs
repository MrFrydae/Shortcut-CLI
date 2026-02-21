use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{
    custom_field_enum_value_json, custom_field_json, full_story_json,
    full_story_json_with_custom_fields, story_custom_field_json,
};
use crate::{UUID_FIELD_1, UUID_VAL_A, UUID_VAL_B, make_create_args, make_update_args};
use sc::{api, commands::story};

#[tokio::test]
async fn get_story_with_custom_fields() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let custom_fields = vec![story_custom_field_json(UUID_FIELD_1, UUID_VAL_A, "High")];
    let body = full_story_json_with_custom_fields(
        42,
        "CF Story",
        "A story with custom fields",
        custom_fields,
    );

    Mock::given(method("GET"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    // Mock custom fields list for name resolution
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
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = story::StoryArgs {
        action: story::StoryAction::Get { id: 42 },
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_with_custom_field() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list for resolution
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
        .expect(1)
        .mount(&server)
        .await;

    let body = full_story_json(200, "CF Create Story", "");

    Mock::given(method("POST"))
        .and(path("/api/v3/stories"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("CF Create Story");
    create_args.custom_fields = vec!["Priority=High".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn update_story_with_custom_field() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list for resolution
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
        .expect(1)
        .mount(&server)
        .await;

    let body = full_story_json(42, "Updated Story", "desc");

    Mock::given(method("PUT"))
        .and(path("/api/v3/stories/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut update_args = make_update_args(42);
    update_args.custom_fields = vec!["Priority=Low".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Update(Box::new(update_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_story_custom_field_invalid_format() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Bad CF Story");
    create_args.custom_fields = vec!["NoEquals".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("FieldName=Value"));
}

#[tokio::test]
async fn create_story_custom_field_unknown_field() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list — field we ask for doesn't exist
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![custom_field_enum_value_json(UUID_VAL_A, "High", 0, true)]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Bad CF Story");
    create_args.custom_fields = vec!["Nonexistent=High".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown custom field"));
    assert!(err.contains("Priority"));
}

#[tokio::test]
async fn create_story_custom_field_unknown_value() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    // Mock custom fields list — value we ask for doesn't exist
    let cf_body = serde_json::json!([custom_field_json(
        UUID_FIELD_1,
        "Priority",
        vec![custom_field_enum_value_json(UUID_VAL_A, "High", 0, true)]
    )]);

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cf_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let mut create_args = make_create_args("Bad CF Story");
    create_args.custom_fields = vec!["Priority=Bogus".to_string()];
    let args = story::StoryArgs {
        action: story::StoryAction::Create(Box::new(create_args)),
    };
    let result = story::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unknown value"));
    assert!(err.contains("High"));
}
