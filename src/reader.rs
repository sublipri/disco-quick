pub use crate::artist::ArtistsReader;
pub use crate::label::LabelsReader;
pub use crate::master::MastersReader;
pub use crate::release::ReleasesReader;
use flate2::read::GzDecoder;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::path::Path;
use thiserror::Error;

pub type XmlReader = quick_xml::Reader<Box<dyn BufRead>>;

pub fn get_xml_reader(path: &Path) -> Result<XmlReader, IoError> {
    let file = File::open(path)?;
    let gz = GzDecoder::new(file);
    let reader: Box<dyn BufRead> = if gz.header().is_some() {
        Box::new(BufReader::new(gz))
    } else {
        let file = File::open(path)?;
        Box::new(BufReader::new(file))
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
    pub fn from_path(path: &Path) -> Result<DiscogsReader, ReaderError> {
        let mut xml_reader = get_xml_reader(path)?;
        let mut buf = Vec::with_capacity(4096);
        let start_event = loop {
            match xml_reader.read_event_into(&mut buf)? {
                Event::Start(ev) => break ev,
                Event::Eof => return Err(ReaderError::NoStartTag),
                _ => continue,
            }
        };
        let reader = match start_event.name().as_ref() {
            b"artists" => DiscogsReader::Artists(Box::new(ArtistsReader::new(xml_reader, buf))),
            b"labels" => DiscogsReader::Labels(Box::new(LabelsReader::new(xml_reader, buf))),
            b"masters" => DiscogsReader::Masters(Box::new(MastersReader::new(xml_reader, buf))),
            b"releases" => DiscogsReader::Releases(Box::new(ReleasesReader::new(xml_reader, buf))),
            _ => {
                return Err(ReaderError::InvalidStartTag);
            }
        };
        Ok(reader)
    }
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error(transparent)]
    IoError(#[from] IoError),
    #[error(transparent)]
    XmlError(#[from] XmlError),
    #[error("No start tag present in file")]
    NoStartTag,
    #[error("Invalid start tag present in file")]
    InvalidStartTag,
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
