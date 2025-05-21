use crate::parser::{Parser, ParserError};
use crate::reader::XmlReader;
use crate::shared::Image;
use crate::util::{find_attr, maybe_text};
use log::debug;
use quick_xml::events::Event;
use std::fmt;
use std::mem::take;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

impl Label {
    pub fn builder(id: u32, name: &str) -> LabelBuilder {
        LabelBuilder {
            inner: Label {
                id,
                name: name.to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Label => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"label" => ParserState::Label,

                Event::Start(e) => match e.local_name().as_ref() {
                    b"name" => ParserState::Name,
                    b"id" => ParserState::Id,
                    b"contactinfo" => ParserState::Contactinfo,
                    b"profile" => ParserState::Profile,
                    b"parentLabel" => {
                        self.current_parent_id = Some(find_attr(e, b"id")?.parse()?);
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
                    let image = Image::from_event(e)?;
                    self.current_item.images.push(image);
                    ParserState::Images
                }
                Event::End(e) if e.local_name().as_ref() == b"images" => ParserState::Label,

                _ => ParserState::Images,
            },

            ParserState::Contactinfo => match ev {
                Event::Text(e) => {
                    self.current_item.contactinfo = maybe_text(e)?;
                    ParserState::Contactinfo
                }
                _ => ParserState::Label,
            },

            ParserState::Profile => match ev {
                Event::Text(e) => {
                    self.current_item.profile = maybe_text(e)?;
                    ParserState::Profile
                }
                _ => ParserState::Label,
            },

            ParserState::ParentLabel => match ev {
                Event::Text(e) => {
                    let Some(id) = self.current_parent_id else {
                        return Err(ParserError::MissingData("Label parent ID"));
                    };
                    let parent_label = LabelInfo {
                        id,
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
                    self.current_sublabel_id = Some(find_attr(e, b"id")?.parse()?);
                    ParserState::Sublabel
                }
                Event::End(e) if e.local_name().as_ref() == b"sublabels" => ParserState::Label,

                _ => ParserState::Sublabels,
            },

            ParserState::Sublabel => match ev {
                Event::Text(e) => {
                    let Some(id) = self.current_sublabel_id else {
                        return Err(ParserError::MissingData("Label sublabel ID"));
                    };
                    let sublabel = LabelInfo {
                        id,
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

pub struct LabelBuilder {
    inner: Label,
}

impl LabelBuilder {
    pub fn id(mut self, id: u32) -> Self {
        self.inner.id = id;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.inner.name = name.to_string();
        self
    }

    pub fn contactinfo(mut self, contactinfo: &str) -> Self {
        self.inner.contactinfo = Some(contactinfo.to_string());
        self
    }

    pub fn profile(mut self, profile: &str) -> Self {
        self.inner.profile = Some(profile.to_string());
        self
    }

    pub fn parent_label(mut self, id: u32, name: &str) -> Self {
        self.inner.parent_label = Some(LabelInfo {
            id,
            name: name.to_string(),
        });
        self
    }

    pub fn sublabel(mut self, id: u32, name: &str) -> Self {
        self.inner.sublabels.push(LabelInfo {
            id,
            name: name.to_string(),
        });
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.inner.urls.push(url.to_string());
        self
    }

    pub fn data_quality(mut self, data_quality: &str) -> Self {
        self.inner.data_quality = data_quality.to_string();
        self
    }

    pub fn image(mut self, ty: &str, width: i16, height: i16) -> Self {
        self.inner.images.push(Image {
            r#type: ty.to_string(),
            uri: None,
            uri150: None,
            width,
            height,
        });
        self
    }

    pub fn build(self) -> Label {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::{BufRead, BufReader, Cursor};

    use super::{Label, LabelsReader};

    fn parse(xml: &'static str) -> Label {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(Cursor::new(xml)));
        let mut reader = quick_xml::Reader::from_reader(reader);
        reader.config_mut().trim_text(true);
        LabelsReader::new(reader, Vec::new()).next().unwrap()
    }

    #[test]
    fn test_label_1000_20231001() {
        let expected = Label::builder(1000, "Warner Bros. Records")
            .contactinfo("3300 Warner Boulevard\r\nBurbank, CA 91505-4964\r\nUSA")
            .profile("Label Code: LC 0392 / LC 00392\r\n\r\nFounded in 1958 by Jack Warner as a soundtrack factory for Warner Bros. movie studios, Warner Bros. Records and its family of subsidiary labels, which includes Reprise Records, Sire Records, Maverick Records, Warner Nashville, Warner Jazz, Warner Western, and Word Label Group encompassed a full spectrum of musical genres.\r\nAfter more than 60 years using the Warner Bros. name and logo (and following the end of a 15-year licensing agreement with AT&T/WarnerMedia, until 2018 Time Warner), the label was rebranded in May 2019 to simply [l=Warner Records].")
            .data_quality("Needs Vote")
            .parent_label(90718, "Warner Bros. Records Inc.")
            .sublabel(29742, "Warner Resound")
            .sublabel(41256, "Warner Special Products")
            .url("http://www.warnerrecords.com")
            .url("http://myspace.com/warnerbrosrecords")
            .image("primary", 600, 818)
            .image("secondary", 600, 600)
            .build();
        let parsed = parse(
            r#"
<label>
  <images>
    <image type="primary" uri="" uri150="" width="600" height="818"/>
    <image type="secondary" uri="" uri150="" width="600" height="600"/>
  </images>
  <id>1000</id>
  <name>Warner Bros. Records</name>
  <contactinfo>3300 Warner Boulevard&#13;
Burbank, CA 91505-4964&#13;
USA</contactinfo>
  <profile>Label Code: LC 0392 / LC 00392&#13;
&#13;
Founded in 1958 by Jack Warner as a soundtrack factory for Warner Bros. movie studios, Warner Bros. Records and its family of subsidiary labels, which includes Reprise Records, Sire Records, Maverick Records, Warner Nashville, Warner Jazz, Warner Western, and Word Label Group encompassed a full spectrum of musical genres.&#13;
After more than 60 years using the Warner Bros. name and logo (and following the end of a 15-year licensing agreement with AT&amp;T/WarnerMedia, until 2018 Time Warner), the label was rebranded in May 2019 to simply [l=Warner Records].</profile>
  <data_quality>Needs Vote</data_quality>
  <parentLabel id="90718">Warner Bros. Records Inc.</parentLabel>
  <urls>
    <url>http://www.warnerrecords.com</url>
    <url>http://myspace.com/warnerbrosrecords</url>
  </urls>
  <sublabels>
    <label id="29742">Warner Resound</label>
    <label id="41256">Warner Special Products</label>
  </sublabels>
</label> "#,
        );
        assert_eq!(expected, parsed);
    }
}
