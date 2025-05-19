use crate::artist_credit::{
    get_credit_string, ArtistCredit, ArtistCreditBuilder, ArtistCreditParser,
};
use crate::company::{CompanyParser, ReleaseCompany};
use crate::parser::{Parser, ParserError};
use crate::reader::XmlReader;
use crate::shared::Image;
use crate::track::{Track, TrackParser};
use crate::util::{find_attr, find_attr_optional, maybe_text};
use crate::video::{Video, VideoParser};
use log::debug;
use quick_xml::events::Event;
use std::fmt;
use std::mem::take;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Release {
    pub id: u32,
    pub status: String,
    pub title: String,
    pub artists: Vec<ArtistCredit>,
    pub country: String,
    pub labels: Vec<ReleaseLabel>,
    pub series: Vec<ReleaseLabel>,
    pub released: String,
    pub notes: Option<String>,
    pub genres: Vec<String>,
    pub styles: Vec<String>,
    pub master_id: Option<u32>,
    pub is_main_release: bool,
    pub data_quality: String,
    pub images: Vec<Image>,
    pub videos: Vec<Video>,
    pub extraartists: Vec<ArtistCredit>,
    pub tracklist: Vec<Track>,
    pub formats: Vec<ReleaseFormat>,
    pub companies: Vec<ReleaseCompany>,
    pub identifiers: Vec<ReleaseIdentifier>,
}

impl Release {
    pub fn builder(id: u32, title: &str) -> ReleaseBuilder {
        ReleaseBuilder {
            inner: Release {
                id,
                title: title.to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseLabel {
    pub id: Option<u32>,
    pub name: String,
    pub catno: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseFormat {
    pub qty: String, // https://www.discogs.com/release/8262262
    pub name: String,
    pub text: Option<String>,
    pub descriptions: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReleaseIdentifier {
    pub r#type: String,
    pub description: Option<String>,
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
    Series,
    Videos,
    Artists,
    ExtraArtists,
    TrackList,
    Format,
    Companies,
    Identifiers,
    Images,
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
                    self.current_item.id = find_attr(e, b"id")?.parse()?;
                    debug!("Began parsing Release {}", self.current_item.id);
                    self.current_item.status = find_attr(e, b"status")?.to_string();
                    ParserState::Release
                }
                Event::Start(e) if e.local_name().as_ref() == b"master_id" => {
                    self.current_item.is_main_release =
                        find_attr(e, b"is_main_release")?.parse()?;
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
                    b"series" => ParserState::Series,
                    b"videos" => ParserState::Videos,
                    b"artists" => ParserState::Artists,
                    b"extraartists" => ParserState::ExtraArtists,
                    b"tracklist" => ParserState::TrackList,
                    b"formats" => ParserState::Format,
                    b"identifiers" => ParserState::Identifiers,
                    b"companies" => ParserState::Companies,
                    b"images" => ParserState::Images,
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
                    self.current_item.notes = maybe_text(e)?;
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
                    let format = ReleaseFormat {
                        name: find_attr(e, b"name")?.to_string(),
                        qty: find_attr(e, b"qty")?.to_string(),
                        text: find_attr_optional(e, b"text")?.map(|t| t.to_string()),
                        ..Default::default()
                    };
                    self.current_item.formats.push(format);
                    ParserState::Format
                }
                Event::Text(e) => {
                    let description = e.unescape()?.to_string();
                    let Some(format) = self.current_item.formats.last_mut() else {
                        return Err(ParserError::MissingData("Release format"));
                    };
                    format.descriptions.push(description);
                    ParserState::Format
                }
                Event::End(e) if e.local_name().as_ref() == b"formats" => ParserState::Release,

                _ => ParserState::Format,
            },

            ParserState::Identifiers => match ev {
                Event::Empty(e) => {
                    let identifier = ReleaseIdentifier {
                        r#type: find_attr(e, b"type")?.to_string(),
                        description: find_attr_optional(e, b"description")?.map(|d| d.to_string()),
                        value: find_attr_optional(e, b"value")?.map(|v| v.to_string()),
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
                    let id = match find_attr_optional(e, b"id")? {
                        Some(id) => Some(id.parse()?),
                        None => None,
                    };
                    let label = ReleaseLabel {
                        name: find_attr(e, b"name")?.to_string(),
                        catno: find_attr_optional(e, b"catno")?.map(|c| c.to_string()),
                        id,
                    };
                    self.current_item.labels.push(label);
                    ParserState::Labels
                }
                _ => ParserState::Release,
            },

            ParserState::Series => match ev {
                Event::Empty(e) => {
                    let id = match find_attr_optional(e, b"id")? {
                        Some(id) => Some(id.parse()?),
                        None => None,
                    };
                    let series = ReleaseLabel {
                        name: find_attr(e, b"name")?.to_string(),
                        catno: find_attr_optional(e, b"catno")?.map(|c| c.to_string()),
                        id,
                    };
                    self.current_item.series.push(series);
                    ParserState::Series
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

            ParserState::Images => match ev {
                Event::Empty(e) if e.local_name().as_ref() == b"image" => {
                    let image = Image::from_event(e)?;
                    self.current_item.images.push(image);
                    ParserState::Images
                }
                Event::End(e) if e.local_name().as_ref() == b"images" => ParserState::Release,

                _ => ParserState::Images,
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

pub struct ReleaseBuilder {
    inner: Release,
}

impl ReleaseBuilder {
    pub fn id(mut self, id: u32) -> Self {
        self.inner.id = id;
        self
    }

    pub fn status(mut self, status: &str) -> Self {
        self.inner.status = status.to_string();
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.inner.title = title.to_string();
        self
    }

    pub fn artist(mut self, credit: ArtistCredit) -> Self {
        self.inner.artists.push(credit);
        self
    }

    pub fn country(mut self, country: &str) -> Self {
        self.inner.country = country.to_string();
        self
    }

    pub fn label(mut self, id: Option<u32>, name: &str, catno: Option<&str>) -> Self {
        self.inner.labels.push(ReleaseLabel {
            id,
            name: name.to_string(),
            catno: catno.map(|c| c.to_string()),
        });
        self
    }

    pub fn series(mut self, id: Option<u32>, name: &str, catno: Option<&str>) -> Self {
        self.inner.series.push(ReleaseLabel {
            id,
            name: name.to_string(),
            catno: catno.map(|c| c.to_string()),
        });
        self
    }

    pub fn released(mut self, released: &str) -> Self {
        self.inner.released = released.to_string();
        self
    }

    pub fn notes(mut self, notes: &str) -> Self {
        self.inner.notes = Some(notes.to_string());
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

    pub fn master_id(mut self, id: u32) -> Self {
        self.inner.master_id = Some(id);
        self
    }

    pub fn is_main_release(mut self, is: bool) -> Self {
        self.inner.is_main_release = is;
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

    pub fn extraartist(mut self, builder: ArtistCreditBuilder) -> Self {
        self.inner.extraartists.push(builder.build());
        self
    }

    pub fn track(self, position: &str, title: &str) -> TrackBuilder {
        TrackBuilder {
            inner: Track {
                position: position.to_string(),
                title: title.to_string(),
                ..Default::default()
            },
            release: self,
        }
    }

    pub fn format(
        mut self,
        qty: &str,
        name: &str,
        text: Option<&str>,
        descriptions: &[&'static str],
    ) -> Self {
        self.inner.formats.push(ReleaseFormat {
            qty: qty.to_string(),
            name: name.to_string(),
            text: text.map(|t| t.to_string()),
            descriptions: descriptions.iter().map(|d| d.to_string()).collect(),
        });
        self
    }

    pub fn company(
        mut self,
        id: u32,
        name: &str,
        catno: Option<&str>,
        entity_type: u8,
        entity_type_name: &str,
    ) -> Self {
        self.inner.companies.push(ReleaseCompany {
            id: Some(id),
            name: name.to_string(),
            catno: catno.map(|c| c.to_string()),
            entity_type,
            entity_type_name: entity_type_name.to_string(),
        });
        self
    }

    pub fn identifier(mut self, ty: &str, description: Option<&str>, value: Option<&str>) -> Self {
        self.inner.identifiers.push(ReleaseIdentifier {
            r#type: ty.to_string(),
            description: description.map(|d| d.to_string()),
            value: value.map(|v| v.to_string()),
        });
        self
    }

    pub fn build(self) -> Release {
        self.inner
    }
}

pub struct TrackBuilder {
    inner: Track,
    release: ReleaseBuilder,
}

impl TrackBuilder {
    pub fn duration(mut self, duration: &str) -> Self {
        self.inner.duration = Some(duration.to_string());
        self
    }
    pub fn artist(mut self, credit: ArtistCreditBuilder) -> Self {
        self.inner.artists.push(credit.build());
        self
    }

    pub fn extraartist(mut self, credit: ArtistCreditBuilder) -> Self {
        self.inner.extraartists.push(credit.build());
        self
    }

    pub fn build_track(mut self) -> ReleaseBuilder {
        self.release.inner.tracklist.push(self.inner);
        self.release
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::io::{BufRead, BufReader, Cursor};

    use crate::artist_credit::{ArtistCredit, ArtistCreditBuilder};

    use super::{Release, ReleasesReader};

    fn parse(xml: &'static str) -> Release {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(Cursor::new(xml)));
        let mut reader = quick_xml::Reader::from_reader(reader);
        reader.config_mut().trim_text(true);
        let mut labels = ReleasesReader::new(reader, Vec::new());
        labels.next().unwrap()
    }

    fn credit(id: u32, name: &str) -> ArtistCreditBuilder {
        ArtistCredit::builder(id, name)
    }

    #[test]
    fn test_release_40299_20250501() {
        let expected = Release::builder(40299, "New Beat - Take 4")
            .artist(credit(194, "Various").build())
            .country("Belgium")
            .status("Accepted")
            .label(Some(9789), "Subway Dance", Some("Subway Dance 4000"))
            .label(Some(9789), "Subway Dance", Some("SD 4000-LP"))
            .series(Some(183060), "Take", Some("4"))
            .series(Some(475876), "A.B.-Sounds", None)
            .released("1989")
            .notes("Made in Belgium.")
            .genre("Electronic")
            .style("Acid")
            .style("New Beat")
            .master_id(35574)
            .is_main_release(true)
            .data_quality("Needs Vote")
            .video("https://www.youtube.com/watch?v=Txq736EVa80", 181, "Tragic Error - Tanzen (1989)", "A Belgian New Beat classic!\r\n\r\nTrack produced and written by Patrick De Meyer.")
            .video("https://www.youtube.com/watch?v=6KwqUVPJ-xc", 303, "Westbam-Monkey say monkey do", "Classic house from 1988,Label-Dance Trax,catalog#: DRX 612,format 12\" vinyl Germany 1988")
            .extraartist(
                credit(118541, "Maurice Engelen")
                    .anv("The Maurice Engelen")
                    .role("Compiled By"),
            )
            .extraartist(credit(501662, "Tejo De Roeck").role("Cover"))
            .extraartist(credit(11701904, "Boy Toy (6)").role("Model"))
            .extraartist(credit(3601091, "Annick Wets").role("Photography By [Photo]"))
            .track("A1", "Tanzen")
            .duration("3:37")
            .artist(credit(7542, "Tragic Error"))
            .extraartist(
                credit(116415, "Patrick De Meyer")
                    .anv("P. De Meyer")
                    .role("Written-By"),
            )
            .build_track()
            .track("A2", "New Beat, A Musical Phenomenon")
            .duration("3:40")
            .artist(credit(32087, "The Brotherhood Of Sleep"))
            .extraartist(credit(221853, "Joey Morton").anv("Morton").role("Written-By"))
            .extraartist(credit(25528, "Sherman").role("Written-By"))
            .build_track()
            .format("1", "Vinyl", None, &["LP", "Compilation"])
            .company(216650, "BE's Songs", None, 21, "Published By")
            .company(57563, "Music Man Import", None, 21, "Published By")
            .identifier("Rights Society", None, Some("SABAM-BIEM"))
            .identifier("Matrix / Runout", Some("Side A"), Some("SD 4000-A2"))
            .identifier("Matrix / Runout", Some("Side B"), Some("SD 4000-B1 FOON"))
            .build();

        let parsed = parse(
            r#"
<release id="40299" status="Accepted">
  <artists>
    <artist>
      <id>194</id>
      <name>Various</name>
    </artist>
  </artists>
  <title>New Beat - Take 4</title>
  <labels>
    <label name="Subway Dance" catno="Subway Dance 4000" id="9789"/>
    <label name="Subway Dance" catno="SD 4000-LP" id="9789"/>
  </labels>
  <series>
    <series name="Take" catno="4" id="183060"/>
    <series name="A.B.-Sounds" catno="" id="475876"/>
  </series>
  <extraartists>
    <artist>
      <id>118541</id>
      <name>Maurice Engelen</name>
      <anv>The Maurice Engelen</anv>
      <role>Compiled By</role>
    </artist>
    <artist>
      <id>501662</id>
      <name>Tejo De Roeck</name>
      <role>Cover</role>
    </artist>
    <artist>
      <id>11701904</id>
      <name>Boy Toy (6)</name>
      <role>Model</role>
    </artist>
    <artist>
      <id>3601091</id>
      <name>Annick Wets</name>
      <role>Photography By [Photo]</role>
    </artist>
  </extraartists>
  <formats>
    <format name="Vinyl" qty="1" text="">
      <descriptions>
        <description>LP</description>
        <description>Compilation</description>
      </descriptions>
    </format>
  </formats>
  <genres>
    <genre>Electronic</genre>
  </genres>
  <styles>
    <style>Acid</style>
    <style>New Beat</style>
  </styles>
  <country>Belgium</country>
  <released>1989</released>
  <notes>Made in Belgium.</notes>
  <data_quality>Needs Vote</data_quality>
  <master_id is_main_release="true">35574</master_id>
  <tracklist>
    <track>
      <position>A1</position>
      <title>Tanzen</title>
      <duration>3:37</duration>
      <artists>
        <artist>
          <id>7542</id>
          <name>Tragic Error</name>
        </artist>
      </artists>
      <extraartists>
        <artist>
          <id>116415</id>
          <name>Patrick De Meyer</name>
          <anv>P. De Meyer</anv>
          <role>Written-By</role>
        </artist>
      </extraartists>
    </track>
    <track>
      <position>A2</position>
      <title>New Beat, A Musical Phenomenon</title>
      <duration>3:40</duration>
      <artists>
        <artist>
          <id>32087</id>
          <name>The Brotherhood Of Sleep</name>
        </artist>
      </artists>
      <extraartists>
        <artist>
          <id>221853</id>
          <name>Joey Morton</name>
          <anv>Morton</anv>
          <role>Written-By</role>
        </artist>
        <artist>
          <id>25528</id>
          <name>Sherman</name>
          <role>Written-By</role>
        </artist>
      </extraartists>
    </track>
  </tracklist>
  <identifiers>
    <identifier type="Rights Society" description="" value="SABAM-BIEM"/>
    <identifier type="Matrix / Runout" description="Side A" value="SD 4000-A2"/>
    <identifier type="Matrix / Runout" description="Side B" value="SD 4000-B1 FOON"/>
  </identifiers>
  <videos>
    <video src="https://www.youtube.com/watch?v=Txq736EVa80" duration="181" embed="true">
      <title>Tragic Error - Tanzen (1989)</title>
      <description>A Belgian New Beat classic!&#13;
&#13;
Track produced and written by Patrick De Meyer.</description>
    </video>
    <video src="https://www.youtube.com/watch?v=6KwqUVPJ-xc" duration="303" embed="true">
      <title>Westbam-Monkey say monkey do</title>
      <description>Classic house from 1988,Label-Dance Trax,catalog#: DRX 612,format 12" vinyl Germany 1988</description>
    </video>
  </videos>
  <companies>
    <company>
      <id>216650</id>
      <name>BE's Songs</name>
      <entity_type>21</entity_type>
      <entity_type_name>Published By</entity_type_name>
      <resource_url>https://api.discogs.com/labels/216650</resource_url>
    </company>
    <company>
      <id>57563</id>
      <name>Music Man Import</name>
      <entity_type>21</entity_type>
      <entity_type_name>Published By</entity_type_name>
      <resource_url>https://api.discogs.com/labels/57563</resource_url>
    </company>
  </companies>
</release>
        "#,
        );
        assert_eq!(expected, parsed)
    }

    #[test]
    fn test_release_40299_20231001() {
        let expected = Release::builder(40299, "New Beat - Take 4")
            .artist(credit(194, "Various").build())
            .country("Belgium")
            .status("Accepted")
            .label(Some(9789), "Subway Dance", Some("Subway Dance 4000"))
            .label(Some(9789), "Subway Dance", Some("SD 4000-LP"))
            .released("1989")
            .notes("Made in Belgium.")
            .genre("Electronic")
            .style("Acid")
            .style("New Beat")
            .master_id(35574)
            .is_main_release(true)
            .data_quality("Needs Vote")
            .video("https://www.youtube.com/watch?v=Txq736EVa80", 181, "Tragic Error - Tanzen (1989)", "A Belgian New Beat classic!\r\n\r\nTrack produced and written by Patrick De Meyer.")
            .extraartist(
                credit(118541, "Maurice Engelen")
                    .anv("The Maurice Engelen")
                    .role("Compiled By"),
            )
            .extraartist(credit(501662, "Tejo De Roeck").role("Cover"))
            .extraartist(credit(11701904, "Boy Toy (6)").role("Model"))
            .extraartist(credit(3601091, "Annick Wets").role("Photography By [Photo]"))
            .track("A1", "Tanzen")
            .duration("3:37")
            .artist(credit(7542, "Tragic Error"))
            .extraartist(
                credit(116415, "Patrick De Meyer")
                    .anv("P. De Meyer")
                    .role("Written-By"),
            )
            .build_track()
            .track("A2", "New Beat, A Musical Phenomenon")
            .duration("3:40")
            .artist(credit(32087, "The Brotherhood Of Sleep"))
            .extraartist(credit(221853, "Joey Morton").anv("Morton").role("Written-By"))
            .extraartist(credit(25528, "Sherman").role("Written-By"))
            .build_track()
            .format("1", "Vinyl", None, &["LP", "Compilation"])
            .company(216650, "BE's Songs", None, 21, "Published By")
            .company(57563, "Music Man Import", None, 21, "Published By")
            .identifier("Rights Society", None, Some("SABAM-BIEM"))
            .identifier("Matrix / Runout", Some("Side A"), Some("SD 4000-A2"))
            .identifier("Matrix / Runout", Some("Side B"), Some("SD 4000-B1 FOON"))
            .image("primary", 600, 595)
            .image("secondary", 600, 614)
            .image("secondary", 589, 600)
            .build();

        let parsed = parse(
            r#"
<release id="40299" status="Accepted">
  <images>
    <image type="primary" uri="" uri150="" width="600" height="595"/>
    <image type="secondary" uri="" uri150="" width="600" height="614"/>
    <image type="secondary" uri="" uri150="" width="589" height="600"/>
  </images>
  <artists>
    <artist>
      <id>194</id>
      <name>Various</name>
      <anv>
      </anv>
      <join>
      </join>
      <role>
      </role>
      <tracks>
      </tracks>
    </artist>
  </artists>
  <title>New Beat - Take 4</title>
  <labels>
    <label name="Subway Dance" catno="Subway Dance 4000" id="9789"/>
    <label name="Subway Dance" catno="SD 4000-LP" id="9789"/>
  </labels>
  <extraartists>
    <artist>
      <id>118541</id>
      <name>Maurice Engelen</name>
      <anv>The Maurice Engelen</anv>
      <join>
      </join>
      <role>Compiled By</role>
      <tracks>
      </tracks>
    </artist>
    <artist>
      <id>501662</id>
      <name>Tejo De Roeck</name>
      <anv>
      </anv>
      <join>
      </join>
      <role>Cover</role>
      <tracks>
      </tracks>
    </artist>
    <artist>
      <id>11701904</id>
      <name>Boy Toy (6)</name>
      <anv>
      </anv>
      <join>
      </join>
      <role>Model</role>
      <tracks>
      </tracks>
    </artist>
    <artist>
      <id>3601091</id>
      <name>Annick Wets</name>
      <anv>
      </anv>
      <join>
      </join>
      <role>Photography By [Photo]</role>
      <tracks>
      </tracks>
    </artist>
  </extraartists>
  <formats>
    <format name="Vinyl" qty="1" text="">
      <descriptions>
        <description>LP</description>
        <description>Compilation</description>
      </descriptions>
    </format>
  </formats>
  <genres>
    <genre>Electronic</genre>
  </genres>
  <styles>
    <style>Acid</style>
    <style>New Beat</style>
  </styles>
  <country>Belgium</country>
  <released>1989</released>
  <notes>Made in Belgium.</notes>
  <data_quality>Needs Vote</data_quality>
  <master_id is_main_release="true">35574</master_id>
  <tracklist>
    <track>
      <position>A1</position>
      <title>Tanzen</title>
      <duration>3:37</duration>
      <artists>
        <artist>
          <id>7542</id>
          <name>Tragic Error</name>
          <anv>
          </anv>
          <join>
          </join>
          <role>
          </role>
          <tracks>
          </tracks>
        </artist>
      </artists>
      <extraartists>
        <artist>
          <id>116415</id>
          <name>Patrick De Meyer</name>
          <anv>P. De Meyer</anv>
          <join>
          </join>
          <role>Written-By</role>
          <tracks>
          </tracks>
        </artist>
      </extraartists>
    </track>
    <track>
      <position>A2</position>
      <title>New Beat, A Musical Phenomenon</title>
      <duration>3:40</duration>
      <artists>
        <artist>
          <id>32087</id>
          <name>The Brotherhood Of Sleep</name>
          <anv>
          </anv>
          <join>
          </join>
          <role>
          </role>
          <tracks>
          </tracks>
        </artist>
      </artists>
      <extraartists>
        <artist>
          <id>221853</id>
          <name>Joey Morton</name>
          <anv>Morton</anv>
          <join>
          </join>
          <role>Written-By</role>
          <tracks>
          </tracks>
        </artist>
        <artist>
          <id>25528</id>
          <name>Sherman</name>
          <anv>
          </anv>
          <join>
          </join>
          <role>Written-By</role>
          <tracks>
          </tracks>
        </artist>
      </extraartists>
    </track>
  </tracklist>
  <identifiers>
    <identifier type="Rights Society" value="SABAM-BIEM"/>
    <identifier type="Matrix / Runout" description="Side A" value="SD 4000-A2"/>
    <identifier type="Matrix / Runout" description="Side B" value="SD 4000-B1 FOON"/>
  </identifiers>
  <videos>
    <video src="https://www.youtube.com/watch?v=Txq736EVa80" duration="181" embed="true">
      <title>Tragic Error - Tanzen (1989)</title>
      <description>A Belgian New Beat classic!&#13;
&#13;
Track produced and written by Patrick De Meyer.</description>
    </video>
  </videos>
  <companies>
    <company>
      <id>216650</id>
      <name>BE's Songs</name>
      <catno>
      </catno>
      <entity_type>21</entity_type>
      <entity_type_name>Published By</entity_type_name>
      <resource_url>https://api.discogs.com/labels/216650</resource_url>
    </company>
    <company>
      <id>57563</id>
      <name>Music Man Import</name>
      <catno>
      </catno>
      <entity_type>21</entity_type>
      <entity_type_name>Published By</entity_type_name>
      <resource_url>https://api.discogs.com/labels/57563</resource_url>
    </company>
  </companies>
</release>
        "#,
        );
        assert_eq!(expected, parsed)
    }
}
