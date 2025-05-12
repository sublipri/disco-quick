use quick_xml::events::{attributes::Attributes, BytesStart};
use std::borrow::Cow;

use crate::parser::ParserError;

pub fn next_attr<'a>(attrs: &mut Attributes<'a>) -> Result<Cow<'a, str>, ParserError> {
    let Some(attr) = attrs.next() else {
        return Err(ParserError::MissingAttr);
    };
    Ok(attr?.unescape_value()?)
}

pub fn get_attr_id(ev: &BytesStart) -> Result<u32, ParserError> {
    let mut attrs = ev.attributes();
    Ok(next_attr(&mut attrs)?.parse()?)
}
