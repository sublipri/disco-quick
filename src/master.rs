use crate::artist_credit::{get_credit_string, ArtistCredit, ArtistCreditParser};
use crate::parser::{Parser, ParserError};
use crate::reader::XmlReader;
use crate::shared::Image;
use crate::util::get_attr_id;
use crate::video::{Video, VideoParser};
use log::debug;
use quick_xml::events::Event;
use std::fmt;
use std::mem::take;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Master {
    pub id: u32,
    pub title: String,
    pub main_release: i32,
    pub year: i32,
    pub notes: Option<String>,
    pub genres: Vec<String>,
    pub styles: Vec<String>,
    pub data_quality: String,
    pub artists: Vec<ArtistCredit>,
    pub images: Vec<Image>,
    pub videos: Vec<Video>,
}

impl fmt::Display for Master {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let artist_credit = get_credit_string(&self.artists);
        write!(f, "{} - {}", artist_credit, self.title)
    }
}

pub struct MastersReader {
    buf: Vec<u8>,
    reader: XmlReader,
    parser: MasterParser,
}

impl MastersReader {
    pub fn new(reader: XmlReader, buf: Vec<u8>) -> Self {
        Self {
            buf,
            reader,
            parser: MasterParser::new(),
        }
    }
}

impl Iterator for MastersReader {
    type Item = Master;
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
    Master,
    MainRelease,
    Artists,
    Title,
    DataQuality,
    Notes,
    Images,
    Styles,
    Genres,
    Year,
    Videos,
}

#[derive(Debug, Default)]
pub struct MasterParser {
    state: ParserState,
    current_item: Master,
    artist_parser: ArtistCreditParser,
    videos_parser: VideoParser,
    item_ready: bool,
}

impl Parser for MasterParser {
    type Item = Master;
    fn new() -> Self {
        Self::default()
    }

    fn take(&mut self) -> Master {
        self.item_ready = false;
        take(&mut self.current_item)
    }

    fn process(&mut self, ev: Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Master => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"master" => {
                    self.current_item.id = get_attr_id(e);
                    debug!("Began parsing Master {}", self.current_item.id);
                    ParserState::Master
                }

                Event::Start(e) => match e.local_name().as_ref() {
                    b"main_release" => ParserState::MainRelease,
                    b"title" => ParserState::Title,
                    b"artists" => ParserState::Artists,
                    b"data_quality" => ParserState::DataQuality,
                    b"images" => ParserState::Images,
                    b"styles" => ParserState::Styles,
                    b"genres" => ParserState::Genres,
                    b"notes" => ParserState::Notes,
                    b"year" => ParserState::Year,
                    b"videos" => ParserState::Videos,
                    _ => ParserState::Master,
                },

                Event::End(e) if e.local_name().as_ref() == b"master" => {
                    self.item_ready = true;
                    ParserState::Master
                }

                _ => ParserState::Master,
            },

            ParserState::MainRelease => match ev {
                Event::Text(e) => {
                    self.current_item.main_release = e.unescape()?.parse()?;
                    ParserState::MainRelease
                }
                _ => ParserState::Master,
            },

            ParserState::Artists => match ev {
                Event::End(e) if e.local_name().as_ref() == b"artists" => ParserState::Master,

                ev => {
                    self.artist_parser.process(ev)?;
                    if self.artist_parser.item_ready {
                        self.current_item.artists.push(self.artist_parser.take());
                    }
                    ParserState::Artists
                }
            },

            ParserState::Title => match ev {
                Event::Text(e) => {
                    self.current_item.title = e.unescape()?.to_string();
                    ParserState::Title
                }
                _ => ParserState::Master,
            },

            ParserState::DataQuality => match ev {
                Event::Text(e) => {
                    self.current_item.data_quality = e.unescape()?.to_string();
                    ParserState::DataQuality
                }
                _ => ParserState::Master,
            },

            ParserState::Images => match ev {
                Event::Empty(e) if e.local_name().as_ref() == b"image" => {
                    let image = Image::from_event(e);
                    self.current_item.images.push(image);
                    ParserState::Images
                }
                Event::End(e) if e.local_name().as_ref() == b"images" => ParserState::Master,

                _ => ParserState::Images,
            },

            ParserState::Genres => match ev {
                Event::End(e) if e.local_name().as_ref() == b"genres" => ParserState::Master,

                Event::Text(e) => {
                    self.current_item.genres.push(e.unescape()?.to_string());
                    ParserState::Genres
                }
                _ => ParserState::Genres,
            },

            ParserState::Styles => match ev {
                Event::End(e) if e.local_name().as_ref() == b"styles" => ParserState::Master,

                Event::Text(e) => {
                    self.current_item.styles.push(e.unescape()?.to_string());
                    ParserState::Styles
                }
                _ => ParserState::Styles,
            },

            ParserState::Notes => match ev {
                Event::Text(e) => {
                    self.current_item.notes = Some(e.unescape()?.to_string());
                    ParserState::Notes
                }
                _ => ParserState::Master,
            },

            ParserState::Year => match ev {
                Event::Text(e) => {
                    self.current_item.year = e.unescape()?.parse()?;
                    ParserState::Year
                }
                _ => ParserState::Master,
            },

            ParserState::Videos => match ev {
                Event::End(e) if e.local_name().as_ref() == b"videos" => ParserState::Master,

                ev => {
                    self.videos_parser.process(ev)?;
                    if self.videos_parser.item_ready {
                        self.current_item.videos.push(self.videos_parser.take());
                    }
                    ParserState::Videos
                }
            },
        };

        Ok(())
    }
}
