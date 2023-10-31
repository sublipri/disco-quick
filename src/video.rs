use crate::parser::{Parser, ParserError};
use crate::util::get_attr;
use quick_xml::events::Event;
use std::mem::take;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Video {
    pub src: String,
    pub duration: u32,
    pub title: String,
    pub description: String,
    pub embed: bool,
}

#[derive(Debug, Default)]
enum ParserState {
    #[default]
    Video,
    Title,
    Description,
}

#[derive(Debug, Default)]
pub struct VideoParser {
    state: ParserState,
    pub current_item: Video,
    pub item_ready: bool,
}

impl Parser for VideoParser {
    type Item = Video;
    fn new() -> Self {
        Self::default()
    }

    fn take(&mut self) -> Video {
        self.item_ready = false;
        take(&mut self.current_item)
    }
    fn process(&mut self, ev: Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Video => match ev {
                Event::Start(e) => match e.local_name().as_ref() {
                    b"video" => {
                        let mut attrs = e.attributes();
                        self.current_item.src = get_attr(attrs.next()).to_string();
                        self.current_item.duration = get_attr(attrs.next()).parse()?;
                        self.current_item.embed = get_attr(attrs.next()).parse()?;
                        ParserState::Video
                    }
                    b"title" => ParserState::Title,
                    b"description" => ParserState::Description,
                    _ => ParserState::Video,
                },

                Event::End(e) => match e.local_name().as_ref() {
                    b"video" => {
                        self.item_ready = true;
                        ParserState::Video
                    }
                    _ => ParserState::Video,
                },

                _ => ParserState::Video,
            },

            ParserState::Title => match ev {
                Event::Text(e) => {
                    self.current_item.title = e.unescape()?.to_string();
                    ParserState::Video
                }
                _ => ParserState::Video,
            },

            ParserState::Description => match ev {
                Event::Text(e) => {
                    self.current_item.description = e.unescape()?.to_string();
                    ParserState::Video
                }
                _ => ParserState::Video,
            },
        };
        Ok(())
    }
}
