use crate::{parser::ParserError, util::next_attr};
use quick_xml::events::BytesStart;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseLabel {
    pub id: u32,
    pub name: String,
    pub catno: Option<String>,
    pub entity_type: u8,
    pub entity_type_name: String,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Image {
    pub r#type: String,
    pub uri: String,
    pub uri150: String,
    pub width: i16,
    pub height: i16,
}

impl Image {
    pub fn from_event(ev: &BytesStart) -> Result<Self, ParserError> {
        let mut attrs = ev.attributes();
        Ok(Image {
            r#type: next_attr(&mut attrs)?.to_string(),
            uri: next_attr(&mut attrs)?.to_string(),
            uri150: next_attr(&mut attrs)?.to_string(),
            width: next_attr(&mut attrs)?.parse()?,
            height: next_attr(&mut attrs)?.parse()?,
        })
    }
}
