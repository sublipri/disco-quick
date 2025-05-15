use log::warn;
use quick_xml::{events::BytesStart, name::QName};
use std::borrow::Cow;

use crate::parser::ParserError;

pub fn find_attr_optional<'a>(
    ev: &'a BytesStart,
    name: &'static [u8],
) -> Result<Option<Cow<'a, str>>, ParserError> {
    for result in ev.attributes() {
        let attr = match result {
            Ok(attr) => attr,
            Err(e) => {
                warn!("Encountered a malformed or duplicate attribute: {e}");
                continue;
            }
        };
        if attr.key == QName(name) {
            if attr.value.is_empty() {
                return Ok(None);
            }
            return Ok(Some(attr.unescape_value()?));
        }
    }
    Ok(None)
}

pub fn find_attr<'a>(ev: &'a BytesStart, name: &'static [u8]) -> Result<Cow<'a, str>, ParserError> {
    find_attr_optional(ev, name)?.ok_or_else(|| {
        let name = unsafe { std::str::from_utf8_unchecked(name) };
        ParserError::MissingAttr(name)
    })
}
