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
pub struct Artist {
    pub id: i32,
    pub name: String,
    pub real_name: Option<String>,
    pub profile: Option<String>,
    pub data_quality: String,
    pub name_variations: Vec<String>,
    pub urls: Vec<String>,
    pub aliases: Vec<ArtistInfo>,
    pub members: Vec<ArtistInfo>,
    pub groups: Vec<ArtistInfo>,
    pub images: Vec<Image>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArtistInfo {
    pub id: u32,
    pub name: String,
}

impl fmt::Display for Artist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct ArtistsReader {
    buf: Vec<u8>,
    reader: XmlReader,
    parser: ArtistParser,
}

impl ArtistsReader {
    pub fn new(reader: XmlReader, buf: Vec<u8>) -> Self {
        Self {
            buf,
            reader,
            parser: ArtistParser::new(),
        }
    }
}

impl Iterator for ArtistsReader {
    type Item = Artist;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_event_into(&mut self.buf).unwrap() {
                Event::Eof => {
                    return None;
                }
                ev => self.parser.process(&ev).unwrap(),
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
    Artist,
    Id,
    Name,
    RealName,
    Profile,
    DataQuality,
    NameVariations,
    Urls,
    Aliases,
    Members,
    MemberId,
    MemberName,
    Groups,
    Images,
}

#[derive(Debug, Default)]
pub struct ArtistParser {
    state: ParserState,
    current_item: Artist,
    item_ready: bool,
}

impl Parser for ArtistParser {
    type Item = Artist;
    fn new() -> Self {
        Self::default()
    }

    fn take(&mut self) -> Self::Item {
        self.item_ready = false;
        take(&mut self.current_item)
    }

    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Artist => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"artist" => ParserState::Artist,

                Event::Start(e) => match e.local_name().as_ref() {
                    b"id" => ParserState::Id,
                    b"name" => ParserState::Name,
                    b"realname" => ParserState::RealName,
                    b"profile" => ParserState::Profile,
                    b"data_quality" => ParserState::DataQuality,
                    b"urls" => ParserState::Urls,
                    b"namevariations" => ParserState::NameVariations,
                    b"aliases" => ParserState::Aliases,
                    b"members" => ParserState::Members,
                    b"groups" => ParserState::Groups,
                    b"images" => ParserState::Images,
                    _ => ParserState::Artist,
                },
                Event::End(e) if e.local_name().as_ref() == b"artist" => {
                    self.item_ready = true;
                    ParserState::Artist
                }
                Event::End(e) if e.local_name().as_ref() == b"artists" => ParserState::Artist,

                _ => ParserState::Artist,
            },

            ParserState::Id => match ev {
                Event::Text(e) => {
                    self.current_item.id = e.unescape()?.parse()?;
                    debug!("Began parsing Artist {}", self.current_item.id);
                    ParserState::Id
                }
                _ => ParserState::Artist,
            },

            ParserState::Name => match ev {
                Event::Text(e) => {
                    self.current_item.name = e.unescape()?.to_string();
                    ParserState::Name
                }
                _ => ParserState::Artist,
            },

            ParserState::RealName => match ev {
                Event::Text(e) => {
                    self.current_item.real_name = Some(e.unescape()?.to_string());
                    ParserState::RealName
                }
                _ => ParserState::Artist,
            },

            ParserState::Profile => match ev {
                Event::Text(e) => {
                    self.current_item.profile = Some(e.unescape()?.to_string());
                    ParserState::Profile
                }
                _ => ParserState::Artist,
            },

            ParserState::DataQuality => match ev {
                Event::Text(e) => {
                    self.current_item.data_quality = e.unescape()?.to_string();
                    ParserState::DataQuality
                }
                _ => ParserState::Artist,
            },

            ParserState::Urls => match ev {
                Event::End(e) if e.local_name().as_ref() == b"urls" => ParserState::Artist,

                Event::Text(e) => {
                    self.current_item.urls.push(e.unescape()?.to_string());
                    ParserState::Urls
                }
                _ => ParserState::Urls,
            },

            ParserState::Aliases => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"name" => {
                    let alias = ArtistInfo {
                        id: get_attr_id(e)?,
                        ..Default::default()
                    };
                    self.current_item.aliases.push(alias);
                    ParserState::Aliases
                }
                Event::Text(e) => {
                    let Some(alias) = self.current_item.aliases.last_mut() else {
                        return Err(ParserError::MissingData);
                    };
                    alias.name = e.unescape()?.to_string();
                    ParserState::Aliases
                }
                Event::End(e) if e.local_name().as_ref() == b"aliases" => ParserState::Artist,

                _ => ParserState::Aliases,
            },

            ParserState::Members => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"name" => ParserState::MemberName,
                Event::Start(e) if e.local_name().as_ref() == b"id" => ParserState::MemberId,
                Event::End(e) if e.local_name().as_ref() == b"members" => ParserState::Artist,
                _ => ParserState::Members,
            },

            ParserState::MemberId => match ev {
                Event::Text(e) => {
                    let member = ArtistInfo {
                        id: e.unescape()?.parse()?,
                        ..Default::default()
                    };
                    self.current_item.members.push(member);
                    ParserState::Members
                }
                _ => ParserState::Members,
            },

            ParserState::MemberName => match ev {
                Event::Text(e) => {
                    let Some(member) = self.current_item.members.last_mut() else {
                        return Err(ParserError::MissingData);
                    };
                    member.name = e.unescape()?.to_string();
                    ParserState::Members
                }
                _ => ParserState::Members,
            },

            ParserState::Groups => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"name" => {
                    let group = ArtistInfo {
                        id: get_attr_id(e)?,
                        ..Default::default()
                    };
                    self.current_item.groups.push(group);
                    ParserState::Groups
                }
                Event::Text(e) => {
                    let Some(group) = self.current_item.groups.last_mut() else {
                        return Err(ParserError::MissingData);
                    };
                    group.name = e.unescape()?.to_string();
                    ParserState::Groups
                }
                Event::End(e) if e.local_name().as_ref() == b"groups" => ParserState::Artist,

                _ => ParserState::Groups,
            },

            ParserState::NameVariations => match ev {
                Event::Text(e) => {
                    let anv = e.unescape()?.to_string();
                    self.current_item.name_variations.push(anv);
                    ParserState::NameVariations
                }
                Event::End(e) if e.local_name().as_ref() == b"namevariations" => {
                    ParserState::Artist
                }
                _ => ParserState::NameVariations,
            },

            ParserState::Images => match ev {
                Event::Empty(e) if e.local_name().as_ref() == b"image" => {
                    let image = Image::from_event(e)?;
                    self.current_item.images.push(image);
                    ParserState::Images
                }
                Event::End(e) if e.local_name().as_ref() == b"images" => ParserState::Artist,

                _ => ParserState::Images,
            },
        };

        Ok(())
    }
}
