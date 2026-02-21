use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::project_json;
use sc::{api, commands::project};

#[tokio::test]
async fn update_project() {
    let out = crate::support::make_output();
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
    let result = project::run(&args, &client, &out).await;
    assert!(result.is_ok());
}
