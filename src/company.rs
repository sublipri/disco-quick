use crate::parser::{Parser, ParserError};
use crate::shared::ReleaseLabel;
use quick_xml::events::Event;
use std::mem::take;

#[derive(Debug, Default)]
pub struct CompanyParser {
    state: ParserState,
    pub current_item: ReleaseLabel,
    pub item_ready: bool,
}

#[derive(Debug, Default)]
enum ParserState {
    #[default]
    Company,
    Id,
    Name,
    Catno,
    EntityType,
    EntityTypeName,
}

impl Parser for CompanyParser {
    type Item = ReleaseLabel;
    fn new() -> Self {
        Self::default()
    }

    fn take(&mut self) -> ReleaseLabel {
        self.item_ready = false;
        take(&mut self.current_item)
    }
    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Company => match ev {
                Event::Start(e) => match e.local_name().as_ref() {
                    b"id" => ParserState::Id,
                    b"name" => ParserState::Name,
                    b"catno" => ParserState::Catno,
                    b"entity_type" => ParserState::EntityType,
                    b"entity_type_name" => ParserState::EntityTypeName,
                    _ => ParserState::Company,
                },

                Event::End(e) if e.local_name().as_ref() == b"company" => {
                    self.item_ready = true;
                    ParserState::Company
                }
                _ => ParserState::Company,
            },

            ParserState::Id => match ev {
                Event::Text(e) => {
                    self.current_item.id = e.unescape()?.parse()?;
                    ParserState::Company
                }
                _ => ParserState::Company,
            },

            ParserState::Name => match ev {
                Event::Text(e) => {
                    self.current_item.name = e.unescape()?.to_string();
                    ParserState::Company
                }
                _ => ParserState::Company,
            },

            ParserState::Catno => match ev {
                Event::Text(e) => {
                    self.current_item.catno = Some(e.unescape()?.to_string());
                    ParserState::Company
                }
                _ => ParserState::Company,
            },

            ParserState::EntityType => match ev {
                Event::Text(e) => {
                    self.current_item.entity_type = e.unescape()?.parse()?;
                    ParserState::Company
                }
                _ => ParserState::Company,
            },

            ParserState::EntityTypeName => match ev {
                Event::Text(e) => {
                    self.current_item.entity_type_name = e.unescape()?.to_string();
                    ParserState::Company
                }
                _ => ParserState::Company,
            },
        };
        Ok(())
    }
}
