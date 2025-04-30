// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

const MAGIC_PREFIX_PNG: &'static str = "iVBORw0KGgo";
const MAGIC_PREFIX_GIF: &'static str = "R0lGOD";
const MAGIC_PREFIX_JPEG: &'static str = "/9j/";

pub fn detect_image_mime_type(base64: &str, mime: Option<mime::Mime>) -> Option<mime::Mime> {
    match mime {
        Some(mime) if mime.type_() == mime::IMAGE => Some(mime),
        None if base64.starts_with(MAGIC_PREFIX_PNG) => Some(mime::IMAGE_PNG),
        None if base64.starts_with(MAGIC_PREFIX_GIF) => Some(mime::IMAGE_GIF),
        None if base64.starts_with(MAGIC_PREFIX_JPEG) => Some(mime::IMAGE_JPEG),
        _ => None,
    }
}
