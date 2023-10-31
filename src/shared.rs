use crate::util::get_attr;
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
    pub fn from_event(ev: BytesStart) -> Self {
        let mut attrs = ev.attributes();
        Image {
            r#type: get_attr(attrs.next()).to_string(),
            uri: get_attr(attrs.next()).to_string(),
            uri150: get_attr(attrs.next()).to_string(),
            width: get_attr(attrs.next()).parse().unwrap(),
            height: get_attr(attrs.next()).parse().unwrap(),
        }
    }
}
