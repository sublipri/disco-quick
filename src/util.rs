use quick_xml::events::{
    attributes::{AttrError, Attribute},
    BytesStart,
};
use std::borrow::Cow;

pub fn get_attr(attr: Option<Result<Attribute<'_>, AttrError>>) -> Cow<'_, str> {
    attr.unwrap().unwrap().unescape_value().unwrap()
}

pub fn get_attr_id(ev: BytesStart) -> u32 {
    let mut attrs = ev.attributes();
    get_attr(attrs.next()).parse().unwrap()
}
