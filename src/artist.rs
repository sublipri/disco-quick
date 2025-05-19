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
pub struct Artist {
    pub id: u32,
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

impl Artist {
    pub fn builder(id: u32, name: &str) -> ArtistBuilder {
        ArtistBuilder {
            inner: Artist {
                id,
                name: name.to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
                    self.current_item.real_name = maybe_text(e)?;
                    ParserState::RealName
                }
                _ => ParserState::Artist,
            },

            ParserState::Profile => match ev {
                Event::Text(e) => {
                    self.current_item.profile = maybe_text(e)?;
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
                        id: find_attr(e, b"id")?.parse()?,
                        ..Default::default()
                    };
                    self.current_item.aliases.push(alias);
                    ParserState::Aliases
                }
                Event::Text(e) => {
                    let Some(alias) = self.current_item.aliases.last_mut() else {
                        return Err(ParserError::MissingData("Artist alias ID"));
                    };
                    alias.name = e.unescape()?.to_string();
                    ParserState::Aliases
                }
                Event::End(e) if e.local_name().as_ref() == b"aliases" => ParserState::Artist,

                _ => ParserState::Aliases,
            },

            ParserState::Members => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"name" => {
                    let member = ArtistInfo {
                        id: find_attr(e, b"id")?.parse()?,
                        ..Default::default()
                    };
                    self.current_item.members.push(member);
                    ParserState::MemberName
                }
                Event::Start(e) if e.local_name().as_ref() == b"id" => ParserState::MemberId,
                Event::End(e) if e.local_name().as_ref() == b"members" => ParserState::Artist,
                _ => ParserState::Members,
            },

            // Removed from the dumps in 2025, but remains present as an attr of the member name
            ParserState::MemberId => match ev {
                Event::Text(_) => ParserState::MemberId,
                _ => ParserState::Members,
            },

            ParserState::MemberName => match ev {
                Event::Text(e) => {
                    let Some(member) = self.current_item.members.last_mut() else {
                        return Err(ParserError::MissingData("Artist member ID"));
                    };
                    member.name = e.unescape()?.to_string();
                    ParserState::Members
                }
                _ => ParserState::Members,
            },

            ParserState::Groups => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"name" => {
                    let group = ArtistInfo {
                        id: find_attr(e, b"id")?.parse()?,
                        ..Default::default()
                    };
                    self.current_item.groups.push(group);
                    ParserState::Groups
                }
                Event::Text(e) => {
                    let Some(group) = self.current_item.groups.last_mut() else {
                        return Err(ParserError::MissingData("Artist group ID"));
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

pub struct ArtistBuilder {
    inner: Artist,
}

impl ArtistBuilder {
    pub fn id(mut self, id: u32) -> Self {
        self.inner.id = id;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.inner.name = name.to_string();
        self
    }

    pub fn real_name(mut self, real_name: &str) -> Self {
        self.inner.real_name = Some(real_name.to_string());
        self
    }

    pub fn profile(mut self, profile: &str) -> Self {
        self.inner.profile = Some(profile.to_string());
        self
    }

    pub fn data_quality(mut self, data_quality: &str) -> Self {
        self.inner.data_quality = data_quality.to_string();
        self
    }

    pub fn name_variation(mut self, name_variation: &str) -> Self {
        self.inner.name_variations.push(name_variation.to_owned());
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.inner.urls.push(url.to_string());
        self
    }

    pub fn alias(mut self, id: u32, name: &str) -> Self {
        self.inner.aliases.push(ArtistInfo {
            id,
            name: name.to_string(),
        });
        self
    }

    pub fn member(mut self, id: u32, name: &str) -> Self {
        self.inner.members.push(ArtistInfo {
            id,
            name: name.to_string(),
        });
        self
    }

    pub fn group(mut self, id: u32, name: &str) -> Self {
        self.inner.groups.push(ArtistInfo {
            id,
            name: name.to_string(),
        });
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

    pub fn build(self) -> Artist {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::{BufRead, BufReader, Cursor};

    use super::{Artist, ArtistsReader};

    fn parse(xml: &'static str) -> Artist {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(Cursor::new(xml)));
        let mut reader = quick_xml::Reader::from_reader(reader);
        reader.config_mut().trim_text(true);
        let mut artists = ArtistsReader::new(reader, Vec::new());
        artists.next().unwrap()
    }

    #[test]
    fn test_artist_2_20231001() {
        let expected = Artist::builder(2, "Mr. James Barth & A.D.")
            .real_name("Cari Lekebusch & Alexi Delano")
            .data_quality("Correct")
            .name_variation("MR JAMES BARTH & A. D.")
            .name_variation("Mr Barth & A.D.")
            .name_variation("Mr. Barth & A.D.")
            .name_variation("Mr. James Barth & A. D.")
            .alias(2470, "Puente Latino")
            .alias(19536, "Yakari & Delano")
            .alias(103709, "Crushed Insect & The Sick Puppy")
            .alias(384581, "ADCL")
            .alias(1779857, "Alexi Delano & Cari Lekebusch")
            .member(26, "Alexi Delano")
            .member(27, "Cari Lekebusch")
            .build();
        let parsed = parse(
            r#"
<artist>
  <id>2</id>
  <name>Mr. James Barth &amp; A.D.</name>
  <realname>Cari Lekebusch &amp; Alexi Delano</realname>
  <profile>
  </profile>
  <data_quality>Correct</data_quality>
  <namevariations>
    <name>MR JAMES BARTH &amp; A. D.</name>
    <name>Mr Barth &amp; A.D.</name>
    <name>Mr. Barth &amp; A.D.</name>
    <name>Mr. James Barth &amp; A. D.</name>
  </namevariations>
  <aliases>
    <name id="2470">Puente Latino</name>
    <name id="19536">Yakari &amp; Delano</name>
    <name id="103709">Crushed Insect &amp; The Sick Puppy</name>
    <name id="384581">ADCL</name>
    <name id="1779857">Alexi Delano &amp; Cari Lekebusch</name>
  </aliases>
  <members>
    <id>26</id>
    <name id="26">Alexi Delano</name>
    <id>27</id>
    <name id="27">Cari Lekebusch</name>
  </members>
</artist>"#,
        );
        assert_eq!(expected, parsed);
    }
    #[test]
    fn test_artist_2_20250501() {
        let expected = Artist::builder(2, "Mr. James Barth & A.D.")
            .data_quality("Correct")
            .name_variation("MR JAMES BARTH & A. D.")
            .name_variation("Mr Barth & A.D.")
            .name_variation("Mr. Barth & A.D.")
            .name_variation("Mr. James Barth & A. D.")
            .alias(2470, "Puente Latino")
            .alias(19536, "Yakari & Delano")
            .alias(103709, "Crushed Insect & The Sick Puppy")
            .alias(384581, "ADCL")
            .alias(1779857, "Alexi Delano & Cari Lekebusch")
            .member(26, "Alexi Delano")
            .member(27, "Cari Lekebusch")
            .build();
        let parsed = parse(
            r#"
<artist>
  <id>2</id>
  <name>Mr. James Barth &amp; A.D.</name>
  <data_quality>Correct</data_quality>
  <namevariations>
    <name>MR JAMES BARTH &amp; A. D.</name>
    <name>Mr Barth &amp; A.D.</name>
    <name>Mr. Barth &amp; A.D.</name>
    <name>Mr. James Barth &amp; A. D.</name>
  </namevariations>
  <aliases>
    <name id="2470">Puente Latino</name>
    <name id="19536">Yakari &amp; Delano</name>
    <name id="103709">Crushed Insect &amp; The Sick Puppy</name>
    <name id="384581">ADCL</name>
    <name id="1779857">Alexi Delano &amp; Cari Lekebusch</name>
  </aliases>
  <members>
    <name id="26">Alexi Delano</name>
    <name id="27">Cari Lekebusch</name>
  </members>
</artist>"#,
        );
        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_artist_26_20231001() {
        let expected = Artist::builder(26, "Alexi Delano")
            .profile("Alexi Delano ‘s music production dwells in perfect balance between shiny minimalism and dark vivacious techno. With more than two decades on stage he has been able to combine different roots and facets of the contemporary music scene.\r\nBorn in Chile, raised in Sweden and later on adopted by New York City, Alexi was part of the Swedish wave of electronic music producers of the mid 90’s such as Adam Beyer, Cari Lekebusch, Jesper Dahlback and Joel Mull. Moving from Scandinavia to New York influenced him to combine the heavy compressed Swedish sound with the vibrancy of the creative music scene of Brooklyn.\r\n\r\nThroughout his music career, Alexi has been nominated for the Swedish Music Award ‘P3 Guld’ (an alternative to the Swedish Grammy), produced six albums and released countless records on established labels such as the iconic Swedish label SVEK, Plus 8, Minus, Hybrid, Drumcode, Visionquest, Spectral Sound, Get Physical, Poker Flat and many more. \r\nWith a music production and DJ style swinging between house and techno, he is consistently reinventing himself with each new release.")
            .data_quality("Needs Vote")
            .name_variation("A Delano")
            .name_variation("A. D.")
            .url("https://www.facebook.com/alexidelanomusic")
            .url("http://www.soundcloud.com/alexidelano")
            .url("http://twitter.com/AlexiDelano")
            .alias(50, "ADNY")
            .alias(937, "G.O.L.")
            .group(2, "Mr. James Barth & A.D.")
            .group(254, "ADNY & The Persuader")
            .image("primary", 600, 269)
            .image("secondary", 600, 400)
            .build();
        let parsed = parse(
            r#"
<artist>
  <images>
    <image type="primary" uri="" uri150="" width="600" height="269"/>
    <image type="secondary" uri="" uri150="" width="600" height="400"/>
  </images>
  <id>26</id>
  <name>Alexi Delano</name>
  <profile>Alexi Delano ‘s music production dwells in perfect balance between shiny minimalism and dark vivacious techno. With more than two decades on stage he has been able to combine different roots and facets of the contemporary music scene.&#13;
Born in Chile, raised in Sweden and later on adopted by New York City, Alexi was part of the Swedish wave of electronic music producers of the mid 90’s such as Adam Beyer, Cari Lekebusch, Jesper Dahlback and Joel Mull. Moving from Scandinavia to New York influenced him to combine the heavy compressed Swedish sound with the vibrancy of the creative music scene of Brooklyn.&#13;
&#13;
Throughout his music career, Alexi has been nominated for the Swedish Music Award ‘P3 Guld’ (an alternative to the Swedish Grammy), produced six albums and released countless records on established labels such as the iconic Swedish label SVEK, Plus 8, Minus, Hybrid, Drumcode, Visionquest, Spectral Sound, Get Physical, Poker Flat and many more. &#13;
With a music production and DJ style swinging between house and techno, he is consistently reinventing himself with each new release.</profile>
  <data_quality>Needs Vote</data_quality>
  <urls>
    <url>https://www.facebook.com/alexidelanomusic</url>
    <url>http://www.soundcloud.com/alexidelano</url>
    <url>http://twitter.com/AlexiDelano</url>
  </urls>
  <namevariations>
    <name>A Delano</name>
    <name>A. D.</name>
  </namevariations>
  <aliases>
    <name id="50">ADNY</name>
    <name id="937">G.O.L.</name>
  </aliases>
  <groups>
    <name id="2">Mr. James Barth &amp; A.D.</name>
    <name id="254">ADNY &amp; The Persuader</name>
  </groups>
</artist>"#,
        );
        assert_eq!(expected, parsed);
    }
}
