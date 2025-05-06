use quick_xml::events::{
    attributes::{AttrError, Attribute},
    BytesStart,
};
use std::borrow::Cow;

pub fn get_attr(attr: Option<Result<Attribute<'_>, AttrError>>) -> Cow<'_, str> {
    if let Some(attr) = attr {
        attr.unwrap().unescape_value().unwrap()
    } else {
        Cow::from("0")
    }
}

pub fn get_attr_id(ev: BytesStart) -> u32 {
    let mut attrs = ev.attributes();
    get_attr(attrs.next()).parse().unwrap()
}
