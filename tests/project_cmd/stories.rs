use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::story_json;
use sc::{api, commands::project};

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
