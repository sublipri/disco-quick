use crate::{
    parser::ParserError,
    util::{find_attr, find_attr_optional},
};
use quick_xml::events::BytesStart;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseLabel {
    pub id: Option<u32>,
    pub name: String,
    pub catno: Option<String>,
    pub entity_type: u8,
    pub entity_type_name: String,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Image {
    pub r#type: String,
    pub uri: Option<String>,
    pub uri150: Option<String>,
    pub width: i16,
    pub height: i16,
}

impl Image {
    pub fn from_event(ev: &BytesStart) -> Result<Self, ParserError> {
        Ok(Image {
            r#type: find_attr(ev, b"type")?.to_string(),
            uri: find_attr_optional(ev, b"uri")?.map(|u| u.to_string()),
            uri150: find_attr_optional(ev, b"uri150")?.map(|u| u.to_string()),
            width: find_attr(ev, b"width")?.parse()?,
            height: find_attr(ev, b"height")?.parse()?,
        })
    }
}
