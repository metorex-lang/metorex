// The clusters a string breaks into, each one character as a reader
// sees it.

use super::*;

/// The extended grapheme clusters a string breaks into, by the rules of
/// Unicode's text segmentation.
pub(crate) fn grapheme_clusters(text: &str) -> Vec<String> {
    use unicode_segmentation::UnicodeSegmentation;
    text.graphemes(true).map(str::to_string).collect()
}

/// The clusters a string breaks into, each tagged with the string's own
/// encoding. An encoding that spells a character in more than one byte is
/// read out of those bytes and written back into them.
pub(crate) fn encoded_grapheme_clusters(string_value: &crate::object::StringValue) -> Vec<Object> {
    use crate::vm::native_methods::string_methods::{
        binary_bytes, wide_bytes, wide_encoding, wide_text,
    };
    let named = string_value.encoding_name();
    if let Some(shape) = wide_encoding(named)
        && !crate::vm::native_methods::string_methods::dummy_encoding(named)
    {
        return grapheme_clusters(&wide_text(&binary_bytes(string_value), shape))
            .iter()
            .map(|cluster| {
                let made = crate::object::StringValue::binary(wide_bytes(cluster, shape));
                made.set_encoding(named);
                Object::String(std::rc::Rc::new(made))
            })
            .collect();
    }
    // An encoding that has one character to the byte has one cluster to the
    // byte with it, since nothing there attaches to anything.
    if matches!(named, "ASCII-8BIT" | "BINARY")
        || crate::vm::native_methods::string_methods::dummy_encoding(named)
        || matches!(named, "Shift_JIS" | "Windows-31J" | "MacJapanese")
    {
        return crate::vm::native_methods::string_methods::encoded_characters(string_value);
    }
    grapheme_clusters(&string_value.as_str())
        .into_iter()
        .map(|cluster| {
            Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
                cluster, named,
            )))
        })
        .collect()
}
