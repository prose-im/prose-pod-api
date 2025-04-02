// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::prelude::*;

// WORKSPACE NAME

api_call_fn!(
    get_workspace_name_unauthenticated,
    unauthenticated: GET,
    "/v1/workspace/name",
);
api_call_fn!(
    set_workspace_name,
    PUT,
    "/v1/workspace/name",
    payload: String,
);

#[given(expr = "the workspace is named {string}")]
async fn given_workspace_name(world: &mut TestWorld, name: String) -> Result<(), Error> {
    world
        .workspace_service()
        .await
        .set_workspace_name(name)
        .await?;
    Ok(())
}

#[when("an unauthenticated user gets the workspace name")]
async fn when_anyone_gets_workspace_name(world: &mut TestWorld) {
    let res = get_workspace_name_unauthenticated(world.api()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the workspace name to {string}")]
async fn when_set_workspace_name(world: &mut TestWorld, name: String, workspace_name: String) {
    let token = user_token!(world, name);
    let res = set_workspace_name(world.api(), token, workspace_name).await;
    world.result = Some(res.into());
}

#[then(expr = "the returned workspace name should be {string}")]
async fn then_response_workspace_name(world: &mut TestWorld, name: String) {
    let res: String = world.result().json();
    assert_eq!(res, name);
}

#[then(expr = "the workspace should be named {string}")]
async fn then_workspace_name(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let workspace_name = world.workspace_service().await.get_workspace_name().await?;
    assert_eq!(workspace_name, name);
    Ok(())
}

// WORKSPACE ICON

api_call_fn!(
    get_workspace_icon_unauthenticated,
    unauthenticated: GET,
    "/v1/workspace/icon",
);
api_call_fn!(set_workspace_icon, PUT, "/v1/workspace/icon", payload: String);

#[given(expr = "the workspace icon is {string}")]
async fn given_workspace_icon(world: &mut TestWorld, png_data: String) -> Result<(), Error> {
    let server_config = world.server_config().await?;
    world.mock_xmpp_service.set_avatar(
        &world.app_config.workspace_jid(&server_config.domain),
        Some(AvatarData::Base64(png_data)),
    )?;
    Ok(())
}

#[when("an unauthenticated user gets the workspace icon")]
async fn when_anyone_gets_workspace_icon(world: &mut TestWorld) {
    let res = get_workspace_icon_unauthenticated(world.api()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the workspace icon to {string}")]
async fn when_set_workspace_icon(world: &mut TestWorld, name: String, png_data: String) {
    let token = user_token!(world, name);
    let res = set_workspace_icon(world.api(), token, png_data).await;
    world.result = Some(res.into());
}

#[then("the returned workspace icon should be undefined")]
async fn then_response_workspace_icon_is_undefined(world: &mut TestWorld) {
    let res: Option<String> = world.result().json();
    assert_eq!(res, None);
}

#[then(expr = "the returned workspace icon should be {string}")]
async fn then_response_workspace_icon(world: &mut TestWorld, png_data: String) {
    let res: Option<String> = world.result().json();
    assert_eq!(res, Some(png_data));
}

#[then(expr = "the workspace icon should be {string}")]
async fn then_workspace_icon(world: &mut TestWorld, png_data: String) -> Result<(), Error> {
    let workspace_icon = world
        .workspace_service()
        .await
        .get_workspace_icon()
        .await?
        .map(|d| d.base64().into_owned());
    assert_eq!(workspace_icon, Some(png_data));
    Ok(())
}

// WORKSPACE ACCENT COLOR

api_call_fn!(
    get_workspace_accent_color_unauthenticated,
    unauthenticated: GET,
    "/v1/workspace/accent-color",
);
api_call_fn!(
    set_workspace_accent_color,
    PUT,
    "/v1/workspace/accent-color",
    payload: String,
);

#[given(expr = "the workspace accent color is {string}")]
async fn given_workspace_accent_color(world: &mut TestWorld, name: String) -> Result<(), Error> {
    world
        .workspace_service()
        .await
        .set_workspace_accent_color(Some(name))
        .await?;
    Ok(())
}

#[when("an unauthenticated user gets the workspace accent color")]
async fn when_anyone_gets_workspace_accent_color(world: &mut TestWorld) {
    let res = get_workspace_accent_color_unauthenticated(world.api()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the workspace accent color to {string}")]
async fn when_set_workspace_accent_color(
    world: &mut TestWorld,
    name: String,
    workspace_name: String,
) {
    let token = user_token!(world, name);
    let res = set_workspace_accent_color(world.api(), token, workspace_name).await;
    world.result = Some(res.into());
}

#[then(expr = "the returned workspace accent color should be {string}")]
async fn then_response_workspace_accent_color(world: &mut TestWorld, accent_color: String) {
    let res: Option<String> = world.result().json();
    assert_eq!(res, Some(accent_color));
}

#[then(expr = "the workspace accent color should be {string}")]
async fn then_workspace_accent_color(
    world: &mut TestWorld,
    accent_color: String,
) -> Result<(), Error> {
    let workspace_accent_color = world
        .workspace_service()
        .await
        .get_workspace_accent_color()
        .await?;
    assert_eq!(workspace_accent_color, Some(accent_color));
    Ok(())
}
