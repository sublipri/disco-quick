use crate::{
    parser::{Parser, ParserError},
    util::maybe_text,
};
use quick_xml::events::Event;
use std::mem::take;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArtistCredit {
    pub id: u32,
    pub name: String,
    pub anv: Option<String>,
    pub join: Option<String>,
    pub role: Option<String>,
    pub tracks: Option<String>,
}

impl ArtistCredit {
    pub fn builder(id: u32, name: &str) -> ArtistCreditBuilder {
        ArtistCreditBuilder {
            inner: ArtistCredit {
                id,
                name: name.to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct ArtistCreditParser {
    state: ParserState,
    pub current_item: ArtistCredit,
    pub item_ready: bool,
}

#[derive(Debug, Default)]
enum ParserState {
    #[default]
    Artist,
    Id,
    Name,
    Anv,
    Join,
    Role,
    Tracks,
}

pub fn get_credit_string(credits: &Vec<ArtistCredit>) -> String {
    if credits.len() == 1 {
        credits[0].name.to_owned()
    } else {
        let mut credit_string = String::new();
        for credit in credits {
            credit_string.push_str(&credit.name);
            if let Some(join) = &credit.join {
                if join != "," {
                    credit_string.push(' ')
                }
                credit_string.push_str(join);
                credit_string.push(' ')
            }
        }
        credit_string
    }
}

impl Parser for ArtistCreditParser {
    type Item = ArtistCredit;
    fn new() -> Self {
        Self {
            state: ParserState::Artist,
            current_item: ArtistCredit::default(),
            item_ready: false,
        }
    }

    fn take(&mut self) -> ArtistCredit {
        self.item_ready = false;
        take(&mut self.current_item)
    }
    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Artist => match ev {
                Event::Start(e) => match e.local_name().as_ref() {
                    b"artist" => ParserState::Artist,
                    b"id" => ParserState::Id,
                    b"name" => ParserState::Name,
                    b"anv" => ParserState::Anv,
                    b"join" => ParserState::Join,
                    b"role" => ParserState::Role,
                    b"tracks" => ParserState::Tracks,
                    _ => ParserState::Artist,
                },
                Event::End(e) if e.local_name().as_ref() == b"artist" => {
                    self.item_ready = true;
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },

            ParserState::Id => match ev {
                Event::Text(e) => {
                    self.current_item.id = e.unescape()?.parse()?;
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },

            ParserState::Name => match ev {
                Event::Text(e) => {
                    self.current_item.name = e.unescape()?.to_string();
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },

            ParserState::Anv => match ev {
                Event::Text(e) => {
                    self.current_item.anv = maybe_text(e)?;
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },

            ParserState::Join => match ev {
                Event::Text(e) => {
                    self.current_item.join = maybe_text(e)?;
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },

            ParserState::Role => match ev {
                Event::Text(e) => {
                    self.current_item.role = maybe_text(e)?;
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },

            ParserState::Tracks => match ev {
                Event::Text(e) => {
                    self.current_item.tracks = maybe_text(e)?;
                    ParserState::Artist
                }
                _ => ParserState::Artist,
            },
        };
        Ok(())
    }
}

pub struct ArtistCreditBuilder {
    inner: ArtistCredit,
}

impl ArtistCreditBuilder {
    pub fn anv(mut self, anv: &str) -> Self {
        self.inner.anv = Some(anv.to_string());
        self
    }
    pub fn join(mut self, join: &str) -> Self {
        self.inner.join = Some(join.to_string());
        self
    }
    pub fn role(mut self, role: &str) -> Self {
        self.inner.role = Some(role.to_string());
        self
    }
    pub fn tracks(mut self, tracks: &str) -> Self {
        self.inner.tracks = Some(tracks.to_string());
        self
    }
    pub fn build(self) -> ArtistCredit {
        self.inner
    }
}
