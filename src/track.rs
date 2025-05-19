use crate::artist_credit::{ArtistCredit, ArtistCreditParser};
use crate::parser::{Parser, ParserError};
use crate::util::maybe_text;
use quick_xml::events::Event;
use std::mem::take;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Track {
    pub position: String,
    pub title: String,
    pub duration: Option<String>,
    pub artists: Vec<ArtistCredit>,
    pub extraartists: Vec<ArtistCredit>,
}

#[derive(Debug, Default)]
enum ParserState {
    #[default]
    Track,
    Position,
    Title,
    Duration,
    Artists,
    ExtraArtists,
}

#[derive(Debug, Default)]
pub struct TrackParser {
    state: ParserState,
    current_item: Track,
    artist_parser: ArtistCreditParser,
    pub item_ready: bool,
}

impl Parser for TrackParser {
    type Item = Track;
    fn new() -> Self {
        TrackParser::default()
    }

    fn take(&mut self) -> Track {
        self.item_ready = false;
        take(&mut self.current_item)
    }

    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Track => match ev {
                Event::Start(e) => match e.local_name().as_ref() {
                    b"track" => ParserState::Track,
                    b"position" => ParserState::Position,
                    b"title" => ParserState::Title,
                    b"duration" => ParserState::Duration,
                    b"artists" => ParserState::Artists,
                    b"extraartists" => ParserState::ExtraArtists,
                    _ => ParserState::Track,
                },
                Event::End(e) if e.local_name().as_ref() == b"track" => {
                    self.item_ready = true;
                    ParserState::Track
                }
                _ => ParserState::Track,
            },

            ParserState::Position => match ev {
                Event::Text(e) => {
                    self.current_item.position = e.unescape()?.to_string();
                    ParserState::Track
                }
                _ => ParserState::Track,
            },

            ParserState::Title => match ev {
                Event::Text(e) => {
                    self.current_item.title = e.unescape()?.to_string();
                    ParserState::Track
                }
                _ => ParserState::Track,
            },

            ParserState::Duration => match ev {
                Event::Text(e) => {
                    self.current_item.duration = maybe_text(e)?;
                    ParserState::Track
                }
                _ => ParserState::Track,
            },

            ParserState::Artists => match ev {
                Event::End(e) if e.local_name().as_ref() == b"artists" => ParserState::Track,

                ev => {
                    self.artist_parser.process(ev)?;
                    if self.artist_parser.item_ready {
                        self.current_item.artists.push(self.artist_parser.take());
                    }
                    ParserState::Artists
                }
            },

            ParserState::ExtraArtists => match ev {
                Event::End(e) if e.local_name().as_ref() == b"extraartists" => ParserState::Track,

                ev => {
                    self.artist_parser.process(ev)?;
                    if self.artist_parser.item_ready {
                        self.current_item
                            .extraartists
                            .push(self.artist_parser.take());
                    }
                    ParserState::ExtraArtists
                }
            },
        };
        Ok(())
    }
}
