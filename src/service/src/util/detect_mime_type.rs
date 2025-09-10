// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub const SUPPORTED_IMAGE_MEDIA_TYPES: [mime::Mime; 3] = [
    mime::IMAGE_PNG,
    mime::IMAGE_GIF,
    mime::IMAGE_JPEG,
];

// NOTE: See [List of file signatures | Wikipedia](https://en.wikipedia.org/wiki/List_of_file_signatures).
const MAGIC_PREFIX_PNG: [u8; 8] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
];
const MAGIC_PREFIX_GIF: [u8; 4] = [
    0x47, 0x49, 0x46, 0x38,
];
const MAGIC_PREFIX_JPEG: [u8; 3] = [0xff, 0xd8, 0xff];

pub fn detect_image_media_type(data: impl AsRef<[u8]>) -> Option<mime::Mime> {
    let data = data.as_ref();
    if data.starts_with(&MAGIC_PREFIX_PNG) {
        Some(mime::IMAGE_PNG)
    } else if data.starts_with(&MAGIC_PREFIX_GIF) {
        Some(mime::IMAGE_GIF)
    } else if data.starts_with(&MAGIC_PREFIX_JPEG) {
        Some(mime::IMAGE_JPEG)
    } else {
        None
    }
}

const MAGIC_PREFIX_PNG_BASE64: &'static str = "iVBORw0KGgo";
const MAGIC_PREFIX_GIF_BASE64: &'static str = "R0lGOD";
const MAGIC_PREFIX_JPEG_BASE64: &'static str = "/9j/";

pub fn detect_image_mime_type_base64(base64: &str) -> Option<mime::Mime> {
    if base64.starts_with(MAGIC_PREFIX_PNG_BASE64) {
        Some(mime::IMAGE_PNG)
    } else if base64.starts_with(MAGIC_PREFIX_GIF_BASE64) {
        Some(mime::IMAGE_GIF)
    } else if base64.starts_with(MAGIC_PREFIX_JPEG_BASE64) {
        Some(mime::IMAGE_JPEG)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::SUPPORTED_IMAGE_MEDIA_TYPES;

    /// We use this in errors, let’s just check the format.
    #[test]
    fn test_supported_types_display() {
        assert_eq!(
            format!("{SUPPORTED_IMAGE_MEDIA_TYPES:?}").as_str(),
            r#"["image/png", "image/gif", "image/jpeg"]"#,
        );
    }
}
