use crate::artist_credit::{
    get_credit_string, ArtistCredit, ArtistCreditBuilder, ArtistCreditParser,
};
use crate::parser::{Parser, ParserError};
use crate::reader::XmlReader;
use crate::shared::Image;
use crate::util::{find_attr, maybe_text};
use crate::video::{Video, VideoParser};
use log::debug;
use quick_xml::events::Event;
use std::fmt;
use std::mem::take;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Master {
    pub id: u32,
    pub title: String,
    pub main_release: u32,
    pub year: u16,
    pub notes: Option<String>,
    pub genres: Vec<String>,
    pub styles: Vec<String>,
    pub data_quality: String,
    pub artists: Vec<ArtistCredit>,
    pub images: Vec<Image>,
    pub videos: Vec<Video>,
}
impl Master {
    pub fn builder(id: u32, title: &str) -> MasterBuilder {
        MasterBuilder {
            inner: Master {
                id,
                title: title.to_string(),
                ..Default::default()
            },
        }
    }
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

    fn process(&mut self, ev: &Event) -> Result<(), ParserError> {
        self.state = match self.state {
            ParserState::Master => match ev {
                Event::Start(e) if e.local_name().as_ref() == b"master" => {
                    self.current_item.id = find_attr(e, b"id")?.parse()?;
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
                    let image = Image::from_event(e)?;
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
                    self.current_item.notes = maybe_text(e)?;
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

pub struct MasterBuilder {
    inner: Master,
}

impl MasterBuilder {
    pub fn id(mut self, id: u32) -> Self {
        self.inner.id = id;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.inner.title = title.to_string();
        self
    }

    pub fn main_release(mut self, id: u32) -> Self {
        self.inner.main_release = id;
        self
    }

    pub fn notes(mut self, notes: &str) -> Self {
        self.inner.notes = Some(notes.to_string());
        self
    }

    pub fn year(mut self, year: u16) -> Self {
        self.inner.year = year;
        self
    }

    pub fn genre(mut self, genre: &str) -> Self {
        self.inner.genres.push(genre.to_string());
        self
    }

    pub fn style(mut self, style: &str) -> Self {
        self.inner.styles.push(style.to_string());
        self
    }

    pub fn data_quality(mut self, data_quality: &str) -> Self {
        self.inner.data_quality = data_quality.to_string();
        self
    }

    pub fn artist(mut self, credit: ArtistCreditBuilder) -> Self {
        self.inner.artists.push(credit.build());
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

    pub fn video(mut self, src: &str, duration: u32, title: &str, description: &str) -> Self {
        self.inner.videos.push(Video {
            src: src.to_string(),
            duration,
            title: title.to_string(),
            description: description.to_string(),
            embed: true,
        });
        self
    }

    pub fn build(self) -> Master {
        self.inner
    }
}
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::{BufRead, BufReader, Cursor};

    use crate::artist_credit::{ArtistCredit, ArtistCreditBuilder};

    use super::{Master, MastersReader};

    fn parse(xml: &'static str) -> Master {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(Cursor::new(xml)));
        let mut reader = quick_xml::Reader::from_reader(reader);
        reader.config_mut().trim_text(true);
        MastersReader::new(reader, Vec::new()).next().unwrap()
    }

    fn credit(id: u32, name: &str) -> ArtistCreditBuilder {
        ArtistCredit::builder(id, name)
    }

    #[test]
    fn test_master_830352_20231001() {
        let expected = Master::builder(830352, "What Is The Time, Mr. Templar? / The Real Jazz")
            .main_release(1671981)
            .year(2009)
            .genre("Electronic")
            .style("Techno")
            .style("Tech House")
            .data_quality("Correct")
            .artist(credit(239, "Jesper Dahlbäck").join("/"))
            .artist(credit(1654, "DK").anv("Dahlbäck & Krome"))
            .image("primary", 600, 600)
            .image("secondary", 600, 600)
            .video(
                "https://www.youtube.com/watch?v=1andhkV72eo",
                311,
                 "JESPER DAHLBÄCK - WHAT IS THE TIME, MR. TEMPLAR? (PND02)",
                 "JESPER DAHLBÄCK - WHAT IS THE TIME, MR. TEMPLAR? (PND02)\n\n#pvnx #thismustbethetrack #trackoftheday #russianadvisor #bvckgrnd #vinyl #vinylcommunity #vinylcollection #recordcollector #vinyladdict #records #music #musiclover #musicblog #vinyloftheday  #mus",
            )
            .video(
                "https://www.youtube.com/watch?v=IWRv_Ye03cU",
                315,
                "J Dahlbäck - The Persuader - What Is The Time, Mr Templar?",
                "From the LP: J. Dahlbäck - The Persuader - Label: Svek - Released: 1997\n\nProblem with the video? Please tell me and it will be removed immediately!"
            )
            .build();
        let parsed = parse(
            r#"
<master id="830352">
  <main_release>1671981</main_release>
  <images>
    <image type="primary" uri="" uri150="" width="600" height="600"/>
    <image type="secondary" uri="" uri150="" width="600" height="600"/>
  </images>
  <artists>
    <artist>
      <id>239</id>
      <name>Jesper Dahlbäck</name>
      <anv>
      </anv>
      <join>/</join>
      <role>
      </role>
      <tracks>
      </tracks>
    </artist>
    <artist>
      <id>1654</id>
      <name>DK</name>
      <anv>Dahlbäck &amp; Krome</anv>
      <join>
      </join>
      <role>
      </role>
      <tracks>
      </tracks>
    </artist>
  </artists>
  <genres>
    <genre>Electronic</genre>
  </genres>
  <styles>
    <style>Techno</style>
    <style>Tech House</style>
  </styles>
  <year>2009</year>
  <title>What Is The Time, Mr. Templar? / The Real Jazz</title>
  <data_quality>Correct</data_quality>
  <videos>
    <video src="https://www.youtube.com/watch?v=1andhkV72eo" duration="311" embed="true">
      <title>JESPER DAHLBÄCK - WHAT IS THE TIME, MR. TEMPLAR? (PND02)</title>
      <description>JESPER DAHLBÄCK - WHAT IS THE TIME, MR. TEMPLAR? (PND02)

#pvnx #thismustbethetrack #trackoftheday #russianadvisor #bvckgrnd #vinyl #vinylcommunity #vinylcollection #recordcollector #vinyladdict #records #music #musiclover #musicblog #vinyloftheday  #mus</description>
    </video>
    <video src="https://www.youtube.com/watch?v=IWRv_Ye03cU" duration="315" embed="true">
      <title>J Dahlbäck - The Persuader - What Is The Time, Mr Templar?</title>
      <description>From the LP: J. Dahlbäck - The Persuader - Label: Svek - Released: 1997

Problem with the video? Please tell me and it will be removed immediately!</description>
    </video>
  </videos>
</master>
        "#,
        );
        assert_eq!(expected, parsed);
    }
}
