// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::TestWorld;
use cucumber::{given, then, when};
use prose_pod_api::error::Error;
use prose_pod_api::features::workspace_details::*;
use rocket::http::{Accept, ContentType};
use rocket::local::asynchronous::{Client, LocalResponse};
use serde_json::json;
use service::models::xmpp::AvatarData;

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
    world
        .workspace_controller()
        .await
        .set_workspace_name(name)
        .await?;
    Ok(())
}

#[when("a user gets the workspace name")]
async fn when_user_gets_workspace_name(world: &mut TestWorld) {
    let res = get_workspace_name(world.client()).await;
    world.result = Some(res.into());
}

#[when(expr = "a user sets the workspace name to {string}")]
async fn when_set_workspace_name(world: &mut TestWorld, name: String) {
    let res = set_workspace_name(world.client(), &name).await;
    world.result = Some(res.into());
}

#[then(expr = "the returned workspace name should be {string}")]
async fn then_response_workspace_name_is(world: &mut TestWorld, name: String) {
    let res: GetWorkspaceNameResponse = world.result().body_into();
    assert_eq!(res.name, name);
}

#[then(expr = "the workspace should be named {string}")]
async fn then_workspace_name_should_be(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let workspace_name = world
        .workspace_controller()
        .await
        .get_workspace_name()
        .await?;
    assert_eq!(workspace_name, name);
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

async fn set_workspace_icon<'a>(client: &'a Client, png_data: String) -> LocalResponse<'a> {
    client
        .put("/v1/workspace/icon")
        .header(Accept::JSON)
        .json(&json!(SetWorkspaceIconRequest { image: png_data }))
        .dispatch()
        .await
}

#[given(expr = "the workspace icon is {string}")]
async fn given_workspace_icon_url(world: &mut TestWorld, png_data: String) -> Result<(), Error> {
    let server_config = world.server_config().await?;
    world.mock_xmpp_service.set_avatar(
        &world.app_config.workspace_jid(&server_config.domain),
        Some(AvatarData::Base64(png_data)),
    )?;
    Ok(())
}

#[when("a user gets the workspace icon")]
async fn when_user_gets_workspace_icon(world: &mut TestWorld) {
    let res = get_workspace_icon(world.client()).await;
    world.result = Some(res.into());
}

#[when(expr = "a user sets the workspace icon to {string}")]
async fn when_set_workspace_icon_url(world: &mut TestWorld, png_data: String) {
    let res = set_workspace_icon(world.client(), png_data).await;
    world.result = Some(res.into());
}

#[then("the returned workspace icon should be undefined")]
async fn then_response_workspace_icon_is_undefined(world: &mut TestWorld) {
    let res: GetWorkspaceIconResponse = world.result().body_into();
    assert_eq!(res.icon, None);
}

#[then(expr = "the returned workspace icon should be {string}")]
async fn then_response_workspace_icon_is(world: &mut TestWorld, png_data: String) {
    let res: GetWorkspaceIconResponse = world.result().body_into();
    assert_eq!(res.icon, Some(png_data));
}

#[then(expr = "the workspace icon should be {string}")]
async fn then_workspace_icon_url_should_be(
    world: &mut TestWorld,
    png_data: String,
) -> Result<(), Error> {
    let workspace_icon = world
        .workspace_controller()
        .await
        .get_workspace_icon()
        .await?
        .map(|d| d.base64().into_owned());
    assert_eq!(workspace_icon, Some(png_data));
    Ok(())
}

// // WORKSPACE ACCENT COLOR

// #[tokio::test]
// async fn test_get_workspace_accent_color_not_initialized() {
//     test_workspace_must_be_initialized(uri!(super::get_workspace_accent_color));
// }

// #[tokio::test]
// async fn test_get_workspace_accent_color() -> Result<(), Box<dyn ErrorError> {
//     let client = rocket_test_client().await;
//     init_workspace(&client).await?;
//     let res: GetWorkspaceAccentColorResponse = get(&client, uri!(super::get_workspace_accent_color))?;

//     assert_eq!(res.color, None);

//     Ok(())
// }

// #[tokio::test]
// async fn test_set_workspace_accent_color() -> Result<(), Box<dyn ErrorError> {
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
