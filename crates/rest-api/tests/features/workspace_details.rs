// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_pod_api::features::workspace_details::*;

use super::prelude::*;

// WORKSPACE NAME

api_call_fn!(
    get_workspace_name_unauthenticated,
    unauthenticated: GET,
    "/v1/workspace/name"
);
api_call_fn!(
    set_workspace_name,
    PUT,
    "/v1/workspace/name",
    SetWorkspaceNameRequest,
    name,
    String
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
async fn when_user_gets_workspace_name(world: &mut TestWorld) {
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
async fn then_response_workspace_name_is(world: &mut TestWorld, name: String) {
    let res: GetWorkspaceNameResponse = world.result().json();
    assert_eq!(res.name, name);
}

#[then(expr = "the workspace should be named {string}")]
async fn then_workspace_name_should_be(world: &mut TestWorld, name: String) -> Result<(), Error> {
    let workspace_name = world.workspace_service().await.get_workspace_name().await?;
    assert_eq!(workspace_name, name);
    Ok(())
}

// WORKSPACE ICON

api_call_fn!(
    get_workspace_icon_unauthenticated,
    unauthenticated: GET,
    "/v1/workspace/icon"
);
api_call_fn!(
    set_workspace_icon,
    PUT,
    "/v1/workspace/icon",
    SetWorkspaceIconRequest,
    image,
    String
);

#[given(expr = "the workspace icon is {string}")]
async fn given_workspace_icon_url(world: &mut TestWorld, png_data: String) -> Result<(), Error> {
    let server_config = world.server_config().await?;
    world.mock_xmpp_service.set_avatar(
        &world.app_config.workspace_jid(&server_config.domain),
        Some(AvatarData::Base64(png_data)),
    )?;
    Ok(())
}

#[when("an unauthenticated user gets the workspace icon")]
async fn when_user_gets_workspace_icon(world: &mut TestWorld) {
    let res = get_workspace_icon_unauthenticated(world.api()).await;
    world.result = Some(res.into());
}

#[when(expr = "{} sets the workspace icon to {string}")]
async fn when_set_workspace_icon_url(world: &mut TestWorld, name: String, png_data: String) {
    let token = user_token!(world, name);
    let res = set_workspace_icon(world.api(), token, png_data).await;
    world.result = Some(res.into());
}

#[then("the returned workspace icon should be undefined")]
async fn then_response_workspace_icon_is_undefined(world: &mut TestWorld) {
    let res: GetWorkspaceIconResponse = world.result().json();
    assert_eq!(res.icon, None);
}

#[then(expr = "the returned workspace icon should be {string}")]
async fn then_response_workspace_icon_is(world: &mut TestWorld, png_data: String) {
    let res: GetWorkspaceIconResponse = world.result().json();
    assert_eq!(res.icon, Some(png_data));
}

#[then(expr = "the workspace icon should be {string}")]
async fn then_workspace_icon_url_should_be(
    world: &mut TestWorld,
    png_data: String,
) -> Result<(), Error> {
    let workspace_icon = world
        .workspace_service()
        .await
        .get_workspace_icon()
        .await?
        .map(|d| d.base64().into_owned());
    assert_eq!(workspace_icon, Some(png_data));
    Ok(())
}
