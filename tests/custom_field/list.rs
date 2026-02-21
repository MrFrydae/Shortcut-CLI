use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{custom_field_enum_value_json, custom_field_json};
use crate::{UUID_FIELD_1, UUID_FIELD_2, UUID_VAL_A, UUID_VAL_B, UUID_VAL_C};
use sc::{api, commands::custom_field};

#[tokio::test]
async fn list_custom_fields() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    let body = serde_json::json!([
        custom_field_json(
            UUID_FIELD_1,
            "Priority",
            vec![
                custom_field_enum_value_json(UUID_VAL_A, "High", 0, true),
                custom_field_enum_value_json(UUID_VAL_B, "Low", 1, true),
            ]
        ),
        custom_field_json(
            UUID_FIELD_2,
            "Risk Level",
            vec![custom_field_enum_value_json(
                UUID_VAL_C, "Critical", 0, true
            )]
        ),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = custom_field::CustomFieldArgs {
        action: custom_field::CustomFieldAction::List,
    };
    let result = custom_field::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_custom_fields_empty() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v3/custom-fields"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = api::client_with_token("test-token", &server.uri()).unwrap();
    let args = custom_field::CustomFieldArgs {
        action: custom_field::CustomFieldAction::List,
    };
    let result = custom_field::run(&args, &client, tmp.path().to_path_buf()).await;
    assert!(result.is_ok());
}
