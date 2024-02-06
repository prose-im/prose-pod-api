// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde_json::json;
use utoipa::openapi::path::{Operation, OperationBuilder};
use utoipa::openapi::request_body::RequestBodyBuilder;
use utoipa::openapi::{
    Content, ContentBuilder, KnownFormat, ObjectBuilder, OneOfBuilder, Ref, ResponseBuilder,
    ResponsesBuilder, Schema, SchemaFormat, SchemaType,
};

pub fn set_workspace_icon() -> Operation {
    let image_content = ContentBuilder::new()
        .schema(
            ObjectBuilder::new()
                .schema_type(SchemaType::String)
                .format(Some(SchemaFormat::KnownFormat(KnownFormat::Binary))),
        )
        .build();
    OperationBuilder::new()
        .tag("Workspace")
        .summary(Some("Set the icon of your workspace."))
        .description(Some("Set the icon of your workspace.\nAccepted content types are: `image/jpeg`, `image/png`, `image/gif`, `image/webp` and `text/plain` (containing either the URL to an external image or a base64-encoded image)."))
        .operation_id(Some("set_workspace_icon"))
        .request_body(Some(RequestBodyBuilder::new()
            .description(Some("Workspace icon"))
            .content("image/jpeg", image_content.clone())
            .content("image/png", image_content.clone())
            .content("image/gif", image_content.clone())
            .content("image/webp", image_content.clone())
            .content("text/plain", ContentBuilder::new()
                .schema(Schema::OneOf(OneOfBuilder::new()
                    .title(Some("URL to an external image or base64-encoded image."))
                    .item(ObjectBuilder::new()
                        .title(Some("URL to an external image"))
                        .schema_type(SchemaType::String)
                        .format(Some(SchemaFormat::Custom("uri".to_string())))
                        .example(Some(json!("https://avatars.githubusercontent.com/u/81181949?s=200&v=4".to_string())))
                        .build()
                    )
                    .item(ObjectBuilder::new()
                        .title(Some("Base64 string"))
                        .schema_type(SchemaType::String)
                        .format(Some(SchemaFormat::Custom("base64".to_string())))
                        .example(Some(json!("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".to_string())))
                        .build()
                    )
                .build()))
            .build())
            .build()
        ))
        .responses(
            ResponsesBuilder::new()
                .response(
                    "200",
                    ResponseBuilder::new()
                    .description("Success")
                    .content("application/json",
                        Content::new(
                            Ref::from_schema_name("GetWorkspaceIconResponse"),
                        ),
                    ),
                ),
        )
        .build()
}
