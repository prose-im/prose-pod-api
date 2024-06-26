// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::TestWorld;
use cucumber::{given, then, when};
use migration::DbErr;
use prose_pod_api::error::Error;
use prose_pod_api::v1::workspace::*;
use rocket::http::{Accept, ContentType};
use rocket::local::asynchronous::{Client, LocalResponse};
use serde_json::json;
use service::sea_orm::{ActiveModelTrait, Set, Unchanged};
use service::Query;

// WORKSPACE NAME

async fn get_workspace_name<'a>(client: &'a Client) -> LocalResponse<'a> {
    client.get("/v1/workspace/name").dispatch().await
}

async fn set_workspace_name<'a>(client: &'a Client, name: &str) -> LocalResponse<'a> {
    client
        .put("/v1/workspace/name")
        .header(ContentType::JSON)
        .body(
            json!(SetWorkspaceNameRequest {
                name: name.to_string(),
            })
            .to_string(),
        )
        .dispatch()
        .await
}

#[given(expr = "the workspace is named {string}")]
async fn given_workspace_name(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let db = world.db();
    let form = entity::workspace::ActiveModel {
        name: Set(name),
        ..Default::default()
    };
    form.save(db).await?;
    Ok(())
}

#[when("a user gets the workspace name")]
async fn when_user_gets_workspace_name(world: &mut TestWorld) {
    let res = get_workspace_name(&world.client).await;
    world.result = Some(res.into());
}

#[when(expr = "a user sets the workspace name to {string}")]
async fn when_set_workspace_name(world: &mut TestWorld, name: String) {
    let res = set_workspace_name(&world.client, &name).await;
    world.result = Some(res.into());
}

#[then(expr = "the returned workspace name should be {string}")]
async fn then_response_workspace_name_is(world: &mut TestWorld, name: String) {
    let res: GetWorkspaceNameResponse = world.result().body_into();
    assert_eq!(res.name, name);
}

#[then(expr = "the workspace should be named {string}")]
async fn then_workspace_name_should_be(world: &mut TestWorld, name: String) -> Result<(), DbErr> {
    let db = world.db();
    let workspace = Query::workspace(db)
        .await?
        .expect("Workspace not initialized");
    assert_eq!(workspace.name, name);
    Ok(())
}

// WORKSPACE ICON

async fn get_workspace_icon<'a>(client: &'a Client) -> LocalResponse<'a> {
    client
        .get("/v1/workspace/icon")
        .header(Accept::JSON)
        .dispatch()
        .await
}

async fn set_workspace_icon_url<'a>(client: &'a Client, url: &str) -> LocalResponse<'a> {
    client
        .put("/v1/workspace/icon")
        .header(Accept::JSON)
        .header(ContentType::Plain)
        .body(url)
        .dispatch()
        .await
}

#[given(expr = "the workspace icon URL is {string}")]
async fn given_workspace_icon_url(world: &mut TestWorld, url: String) -> Result<(), Error> {
    let db = world.db();
    let form = entity::workspace::ActiveModel {
        id: Unchanged(1),
        icon_url: Set(Some(url)),
        ..Default::default()
    };
    form.update(db).await?;
    Ok(())
}

#[when("a user gets the workspace icon")]
async fn when_user_gets_workspace_icon(world: &mut TestWorld) {
    let res = get_workspace_icon(&world.client).await;
    world.result = Some(res.into());
}

#[when(expr = "a user sets the workspace icon URL to {string}")]
async fn when_set_workspace_icon_url(world: &mut TestWorld, url: String) {
    let res = set_workspace_icon_url(&world.client, &url).await;
    world.result = Some(res.into());
}

#[then("the returned workspace icon URL should be undefined")]
async fn then_response_workspace_icon_is_undefined(world: &mut TestWorld) {
    let res: GetWorkspaceIconResponse = world.result().body_into();
    assert_eq!(res.url, None);
}

#[then(expr = "the returned workspace icon URL should be {string}")]
async fn then_response_workspace_icon_is(world: &mut TestWorld, url: String) {
    let res: GetWorkspaceIconResponse = world.result().body_into();
    assert_eq!(res.url, Some(url));
}

#[then(expr = "the workspace icon URL should be {string}")]
async fn then_workspace_icon_url_should_be(
    world: &mut TestWorld,
    url: String,
) -> Result<(), DbErr> {
    let db = world.db();
    let workspace = Query::workspace(db)
        .await?
        .expect("Workspace not initialized");
    assert_eq!(workspace.icon_url, Some(url));
    Ok(())
}

// // WORKSPACE ACCENT COLOR

// #[tokio::test]
// async fn test_get_workspace_accent_color_not_initialized() {
//     test_workspace_must_be_initialized(uri!(super::get_workspace_accent_color));
// }

// #[tokio::test]
// async fn test_get_workspace_accent_color() -> Result<(), Box<dyn Error>> {
//     let client = rocket_test_client().await;
//     init_workspace(&client).await?;
//     let res: GetWorkspaceAccentColorResponse = get(&client, uri!(super::get_workspace_accent_color))?;

//     assert_eq!(res.color, None);

//     Ok(())
// }

// #[tokio::test]
// async fn test_set_workspace_accent_color() -> Result<(), Box<dyn Error>> {
//     let client = rocket_test_client().await;
//     init_workspace(&client).await?;

//     let res: GetWorkspaceAccentColorResponse = get(&client, uri!(super::get_workspace_accent_color))?;
//     assert_eq!(res.color, None);

//     let color = "#4233BE";
//     let res: GetWorkspaceAccentColorResponse = put(
//         &client,
//         uri!(super::set_workspace_accent_color),
//         ContentType::Plain,
//         json!(SetWorkspaceAccentColorRequest {
//             color: color.to_string(),
//         }).to_string(),
//     )?;
//     assert_eq!(res.color, Some(color.to_string()));

//     let res: GetWorkspaceAccentColorResponse = get(&client, uri!(super::get_workspace_accent_color))?;
//     assert_eq!(res.color, Some(color.to_string()));

//     Ok(())
// }
