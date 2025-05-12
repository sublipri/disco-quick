use crate::artist_credit::{get_credit_string, ArtistCredit, ArtistCreditParser};
use crate::company::CompanyParser;
use crate::parser::{Parser, ParserError};
use crate::reader::XmlReader;
use crate::shared::{Image, ReleaseLabel};
use crate::track::{Track, TrackParser};
use crate::util::next_attr;
use crate::video::{Video, VideoParser};
use log::debug;
use quick_xml::events::Event;
use std::fmt;
use std::mem::take;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Release {
    pub id: i32,
    pub status: String,
    pub title: String,
    pub artists: Vec<ArtistCredit>,
    pub country: String,
    pub labels: Vec<ReleaseLabel>,
    pub released: String,
    pub notes: Option<String>,
    pub genres: Vec<String>,
    pub styles: Vec<String>,
    pub master_id: Option<i32>,
    pub is_main_release: bool,
    pub data_quality: String,
    pub images: Vec<Image>,
    pub videos: Vec<Video>,
    pub extraartists: Vec<ArtistCredit>,
    pub tracklist: Vec<Track>,
    pub formats: Vec<ReleaseFormat>,
    pub companies: Vec<ReleaseLabel>,
    pub identifiers: Vec<ReleaseIdentifier>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseFormat {
    pub qty: String, // https://www.discogs.com/release/8262262
    pub name: String,
    pub text: Option<String>,
    pub descriptions: Vec<String>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseIdentifier {
    pub r#type: String,
    pub description: String,
    pub value: Option<String>,
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let artist_credit = get_credit_string(&self.artists);
        write!(f, "{} - {}", artist_credit, self.title)
    }
}

pub struct ReleasesReader {
    buf: Vec<u8>,
    reader: XmlReader,
    parser: ReleaseParser,
}

impl ReleasesReader {
    pub fn new(reader: XmlReader, buf: Vec<u8>) -> Self {
        Self {
            buf,
            reader,
            parser: ReleaseParser::new(),
        }
    }
}

impl Iterator for ReleasesReader {
    type Item = Release;
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
    Release,
    Title,
    Country,
    Released,
    Notes,
    Genres,
    Styles,
    MasterId,
    DataQuality,
    Labels,
    Videos,
    Artists,
    ExtraArtists,
    TrackList,
    Format,
    Companies,
    Identifiers,
}

#[derive(Debug, Default)]
pub struct ReleaseParser {
    state: ParserState,
    current_item: Release,
    artist_parser: ArtistCreditParser,
    video_parser: VideoParser,
    track_parser: TrackParser,
    company_parser: CompanyParser,
    item_ready: bool,
}

impl Parser for ReleaseParser {
    type Item = Release;

    fn new() -> Self {
        Self::default()
    }

    fn take(&mut self) -> Release {
        self.item_ready = false;
        take(&mut self.current_item)
    }

    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Release => match ev {
                Event::End(e) if e.local_name().as_ref() == b"release" => {
                    self.item_ready = true;
                    ParserState::Release
                }
                Event::Start(e) if e.local_name().as_ref() == b"release" => {
                    let mut a = e.attributes();
                    self.current_item.id = next_attr(&mut a)?.parse()?;
                    debug!("Began parsing Release {}", self.current_item.id);
                    self.current_item.status = next_attr(&mut a)?.to_string();
                    ParserState::Release
                }
                Event::Start(e) if e.local_name().as_ref() == b"master_id" => {
                    let mut a = e.attributes();
                    self.current_item.is_main_release = next_attr(&mut a)?.parse()?;
                    ParserState::MasterId
                }
                Event::Start(e) => match e.local_name().as_ref() {
                    b"title" => ParserState::Title,
                    b"country" => ParserState::Country,
                    b"released" => ParserState::Released,
                    b"notes" => ParserState::Notes,
                    b"genres" => ParserState::Genres,
                    b"styles" => ParserState::Styles,
                    b"data_quality" => ParserState::DataQuality,
                    b"labels" => ParserState::Labels,
                    b"videos" => ParserState::Videos,
                    b"artists" => ParserState::Artists,
                    b"extraartists" => ParserState::ExtraArtists,
                    b"tracklist" => ParserState::TrackList,
                    b"formats" => ParserState::Format,
                    b"identifiers" => ParserState::Identifiers,
                    b"companies" => ParserState::Companies,
                    _ => ParserState::Release,
                },
                _ => ParserState::Release,
            },

            ParserState::Title => match ev {
                Event::Text(e) => {
                    self.current_item.title = e.unescape()?.to_string();
                    ParserState::Title
                }
                _ => ParserState::Release,
            },

            ParserState::Country => match ev {
                Event::Text(e) => {
                    self.current_item.country = e.unescape()?.to_string();
                    ParserState::Country
                }
                _ => ParserState::Release,
            },

            ParserState::Released => match ev {
                Event::Text(e) => {
                    self.current_item.released = e.unescape()?.to_string();
                    ParserState::Released
                }
                _ => ParserState::Release,
            },

            ParserState::Notes => match ev {
                Event::Text(e) => {
                    self.current_item.notes = Some(e.unescape()?.to_string());
                    ParserState::Notes
                }
                _ => ParserState::Release,
            },

            ParserState::Artists => match ev {
                Event::End(e) if e.local_name().as_ref() == b"artists" => ParserState::Release,

                ev => {
                    self.artist_parser.process(ev)?;
                    if self.artist_parser.item_ready {
                        self.current_item.artists.push(self.artist_parser.take());
                    }
                    ParserState::Artists
                }
            },

            ParserState::ExtraArtists => match ev {
                Event::End(e) if e.local_name().as_ref() == b"extraartists" => ParserState::Release,

                ev => {
                    self.artist_parser.process(ev)?;
                    if self.artist_parser.item_ready {
                        let ea = self.artist_parser.take();
                        self.current_item.extraartists.push(ea);
                    }
                    ParserState::ExtraArtists
                }
            },

            ParserState::Genres => match ev {
                Event::End(e) if e.local_name().as_ref() == b"genres" => ParserState::Release,

                Event::Text(e) => {
                    self.current_item.genres.push(e.unescape()?.to_string());
                    ParserState::Genres
                }
                _ => ParserState::Genres,
            },

            ParserState::Styles => match ev {
                Event::End(e) if e.local_name().as_ref() == b"styles" => ParserState::Release,

                Event::Text(e) => {
                    self.current_item.styles.push(e.unescape()?.to_string());
                    ParserState::Styles
                }
                _ => ParserState::Styles,
            },

            ParserState::Format => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"format" => {
                    let mut attrs = e.attributes();
                    let mut format = ReleaseFormat {
                        name: next_attr(&mut attrs)?.to_string(),
                        qty: next_attr(&mut attrs)?.to_string(),
                        ..Default::default()
                    };
                    let text = next_attr(&mut attrs)?.to_string();
                    if !text.is_empty() {
                        format.text = Some(text)
                    }
                    self.current_item.formats.push(format);
                    ParserState::Format
                }
                Event::Text(e) => {
                    let description = e.unescape()?.to_string();
                    let Some(format) = self.current_item.formats.last_mut() else {
                        return Err(ParserError::MissingData);
                    };
                    format.descriptions.push(description);
                    ParserState::Format
                }
                Event::End(e) if e.local_name().as_ref() == b"formats" => ParserState::Release,

                _ => ParserState::Format,
            },

            ParserState::Identifiers => match ev {
                Event::Empty(e) => {
                    let mut attrs = e.attributes();
                    let identifier = ReleaseIdentifier {
                        r#type: next_attr(&mut attrs)?.to_string(),
                        description: next_attr(&mut attrs)?.to_string(),
                        value: if let Some(v) = attrs.next() {
                            Some(v?.unescape_value()?.to_string())
                        } else {
                            None
                        },
                    };
                    self.current_item.identifiers.push(identifier);
                    ParserState::Identifiers
                }
                _ => ParserState::Release,
            },

            ParserState::MasterId => match ev {
                Event::Text(e) => {
                    self.current_item.master_id = Some(e.unescape()?.parse()?);
                    ParserState::MasterId
                }
                Event::End(_) => ParserState::Release,

                _ => ParserState::MasterId,
            },

            ParserState::DataQuality => match ev {
                Event::Text(e) => {
                    self.current_item.data_quality = e.unescape()?.to_string();
                    ParserState::DataQuality
                }
                _ => ParserState::Release,
            },

            ParserState::Labels => match ev {
                Event::Empty(e) => {
                    let mut attrs = e.attributes();
                    let label = ReleaseLabel {
                        name: next_attr(&mut attrs)?.to_string(),
                        catno: Some(next_attr(&mut attrs)?.to_string()),
                        id: next_attr(&mut attrs)?.parse()?,
                        entity_type: 1,
                        entity_type_name: "Label".to_string(),
                    };
                    self.current_item.labels.push(label);
                    ParserState::Labels
                }
                _ => ParserState::Release,
            },

            ParserState::Videos => match ev {
                Event::End(e) if e.local_name().as_ref() == b"videos" => ParserState::Release,

                ev => {
                    self.video_parser.process(ev)?;
                    if self.video_parser.item_ready {
                        self.current_item.videos.push(self.video_parser.take());
                    }
                    ParserState::Videos
                }
            },

            ParserState::TrackList => match ev {
                Event::End(e) if e.local_name().as_ref() == b"tracklist" => ParserState::Release,

                ev => {
                    self.track_parser.process(ev)?;
                    if self.track_parser.item_ready {
                        self.current_item.tracklist.push(self.track_parser.take());
                    }
                    ParserState::TrackList
                }
            },

            ParserState::Companies => match ev {
                Event::End(e) if e.local_name().as_ref() == b"companies" => ParserState::Release,

                ev => {
                    self.company_parser.process(ev)?;
                    if self.company_parser.item_ready {
                        self.current_item.companies.push(self.company_parser.take());
                    }
                    ParserState::Companies
                }
            },
        };

        Ok(())
    }
}
