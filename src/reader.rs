pub use crate::artist::ArtistsReader;
pub use crate::label::LabelsReader;
pub use crate::master::MastersReader;
pub use crate::release::ReleasesReader;
use flate2::read::GzDecoder;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError, Seek};
use std::path::Path;
use thiserror::Error;

pub type XmlReader = quick_xml::Reader<Box<dyn BufRead>>;

pub fn get_xml_reader(path: &Path) -> Result<XmlReader, IoError> {
    let file = File::open(path)?;
    let gz = GzDecoder::new(file);
    let reader: Box<dyn BufRead> = if gz.header().is_some() {
        Box::new(BufReader::new(gz))
    } else {
        let mut reader = gz.into_inner();
        reader.rewind()?;
        Box::new(BufReader::new(reader))
    };
    Ok(quick_xml::Reader::from_reader(reader))
}

pub enum DiscogsReader {
    Artists(Box<ArtistsReader>),
    Labels(Box<LabelsReader>),
    Masters(Box<MastersReader>),
    Releases(Box<ReleasesReader>),
}

impl DiscogsReader {
    /// Open an XML file at the given path, and return the appropriate reader based on its contents.
    /// The file can be either uncompressed or gzip compressed.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<DiscogsReader, ReaderError> {
        // Since GzDecoder doesn't impl Seek, we open the file twice. Once to read the start tag,
        // then again so the parsers can read from the start of the file, which is necessary for
        // old versions of the dump that contain e.g. <artist> as the first tag, not <artists>
        let path = path.as_ref();
        let start_tag = {
            let xml_reader = get_xml_reader(path)?;
            read_start_tag(xml_reader)?
        };
        let xml_reader = get_xml_reader(path)?;
        let buf = Vec::with_capacity(4096);
        let reader = match start_tag.as_ref() {
            "artists" | "artist" => {
                DiscogsReader::Artists(Box::new(ArtistsReader::new(xml_reader, buf)))
            }
            "labels" | "label" => {
                DiscogsReader::Labels(Box::new(LabelsReader::new(xml_reader, buf)))
            }
            "masters" | "master" => {
                DiscogsReader::Masters(Box::new(MastersReader::new(xml_reader, buf)))
            }
            "releases" | "release" => {
                DiscogsReader::Releases(Box::new(ReleasesReader::new(xml_reader, buf)))
            }
            _ => {
                return Err(ReaderError::InvalidStartTag(start_tag));
            }
        };
        Ok(reader)
    }
}

fn read_start_tag(mut reader: XmlReader) -> Result<String, ReaderError> {
    let mut buf = Vec::with_capacity(4096);
    let start_event = loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ev) => break ev,
            Event::Eof => return Err(ReaderError::NoStartTag),
            _ => continue,
        }
    };
    Ok(String::from_utf8_lossy(start_event.name().as_ref()).into_owned())
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error(transparent)]
    IoError(#[from] IoError),
    #[error(transparent)]
    XmlError(#[from] XmlError),
    #[error("No start tag present in file")]
    NoStartTag,
    #[error("Invalid start tag present in file: {0}")]
    InvalidStartTag(String),
}

impl fmt::Display for DiscogsReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match &self {
            DiscogsReader::Artists(_) => "artists",
            DiscogsReader::Labels(_) => "labels",
            DiscogsReader::Masters(_) => "masters",
            DiscogsReader::Releases(_) => "releases",
        };
        write!(f, "{name}")
    }
}
