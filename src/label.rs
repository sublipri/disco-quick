use crate::parser::{Parser, ParserError};
use crate::reader::XmlReader;
use crate::shared::Image;
use crate::util::get_attr_id;
use log::debug;
use quick_xml::events::Event;
use std::fmt;
use std::mem::take;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    pub id: u32,
    pub name: String,
    pub contactinfo: Option<String>,
    pub profile: Option<String>,
    pub parent_label: Option<LabelInfo>,
    pub sublabels: Vec<LabelInfo>,
    pub urls: Vec<String>,
    pub data_quality: String,
    pub images: Vec<Image>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabelInfo {
    pub id: u32,
    pub name: String,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct LabelsReader {
    buf: Vec<u8>,
    reader: XmlReader,
    parser: LabelParser,
}

impl LabelsReader {
    pub fn new(reader: XmlReader, buf: Vec<u8>) -> Self {
        Self {
            buf,
            reader,
            parser: LabelParser::new(),
        }
    }
}

impl Iterator for LabelsReader {
    type Item = Label;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_event_into(&mut self.buf).unwrap() {
                Event::Eof => {
                    return None;
                }
                ev => self.parser.process(ev).unwrap(),
            };
            if self.parser.item_ready {
                return Some(self.parser.take());
            }
            self.buf.clear();
        }
    }
}

#[derive(Debug, Default)]
enum ParserState {
    #[default]
    Label,
    Name,
    Id,
    Images,
    Contactinfo,
    Profile,
    ParentLabel,
    Sublabels,
    Sublabel,
    Urls,
    DataQuality,
}

#[derive(Debug, Default)]
pub struct LabelParser {
    state: ParserState,
    current_item: Label,
    current_sublabel_id: Option<u32>,
    current_parent_id: Option<u32>,
    item_ready: bool,
}

impl Parser for LabelParser {
    type Item = Label;
    fn new() -> Self {
        Self::default()
    }
    fn take(&mut self) -> Self::Item {
        self.item_ready = false;
        take(&mut self.current_item)
    }

    fn process(&mut self, ev: Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Label => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"label" => ParserState::Label,

                Event::Start(e) => match e.local_name().as_ref() {
                    b"name" => ParserState::Name,
                    b"id" => ParserState::Id,
                    b"contactinfo" => ParserState::Contactinfo,
                    b"profile" => ParserState::Profile,
                    b"parentLabel" => {
                        self.current_parent_id = Some(get_attr_id(e));
                        ParserState::ParentLabel
                    }
                    b"sublabels" => ParserState::Sublabels,
                    b"urls" => ParserState::Urls,
                    b"images" => ParserState::Images,
                    b"data_quality" => ParserState::DataQuality,
                    _ => ParserState::Label,
                },
                Event::End(e) if e.local_name().as_ref() == b"label" => {
                    self.item_ready = true;
                    ParserState::Label
                }

                _ => ParserState::Label,
            },

            ParserState::Id => match ev {
                Event::Text(e) => {
                    self.current_item.id = e.unescape()?.parse()?;
                    debug!("Began parsing Label {}", self.current_item.id);
                    ParserState::Id
                }
                _ => ParserState::Label,
            },

            ParserState::Name => match ev {
                Event::Text(e) => {
                    self.current_item.name = e.unescape()?.to_string();
                    ParserState::Name
                }
                _ => ParserState::Label,
            },

            ParserState::Images => match ev {
                Event::Empty(e) if e.local_name().as_ref() == b"image" => {
                    let image = Image::from_event(e);
                    self.current_item.images.push(image);
                    ParserState::Images
                }
                Event::End(e) if e.local_name().as_ref() == b"images" => ParserState::Label,

                _ => ParserState::Images,
            },

            ParserState::Contactinfo => match ev {
                Event::Text(e) => {
                    self.current_item.contactinfo = Some(e.unescape()?.to_string());
                    ParserState::Contactinfo
                }
                _ => ParserState::Label,
            },

            ParserState::Profile => match ev {
                Event::Text(e) => {
                    self.current_item.profile = Some(e.unescape()?.to_string());
                    ParserState::Profile
                }
                _ => ParserState::Label,
            },

            ParserState::ParentLabel => match ev {
                Event::Text(e) => {
                    let parent_label = LabelInfo {
                        id: self.current_parent_id.unwrap(),
                        name: e.unescape()?.to_string(),
                    };
                    self.current_item.parent_label = Some(parent_label);
                    self.current_parent_id = None;
                    ParserState::ParentLabel
                }
                _ => ParserState::Label,
            },

            ParserState::Sublabels => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"label" => {
                    self.current_sublabel_id = Some(get_attr_id(e));
                    ParserState::Sublabel
                }
                Event::End(e) if e.local_name().as_ref() == b"sublabels" => ParserState::Label,

                _ => ParserState::Sublabels,
            },

            ParserState::Sublabel => match ev {
                Event::Text(e) => {
                    let sublabel = LabelInfo {
                        id: self.current_sublabel_id.unwrap(),
                        name: e.unescape()?.to_string(),
                    };
                    self.current_item.sublabels.push(sublabel);
                    self.current_sublabel_id = None;
                    ParserState::Sublabels
                }
                _ => ParserState::Sublabels,
            },

            ParserState::Urls => match ev {
                Event::Text(e) => {
                    self.current_item.urls.push(e.unescape()?.to_string());
                    ParserState::Urls
                }
                Event::End(e) if e.local_name().as_ref() == b"urls" => ParserState::Label,

                _ => ParserState::Urls,
            },

            ParserState::DataQuality => match ev {
                Event::Text(e) => {
                    self.current_item.data_quality = e.unescape()?.to_string();
                    ParserState::DataQuality
                }
                _ => ParserState::Label,
            },
        };

        Ok(())
    }
}
