use quick_xml::events::Event;
use thiserror::Error;

pub trait Parser {
    type Item;

    fn new() -> Self;

    fn take(&mut self) -> Self::Item;

    fn process(&mut self, ev: &Event) -> Result<(), ParserError>;
}

#[derive(Error, Debug, Clone)]
pub enum ParserError {
    #[error(transparent)]
    QuickXml(#[from] quick_xml::Error),
    #[error(transparent)]
    QuickXmlAttr(#[from] quick_xml::events::attributes::AttrError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseBool(#[from] std::str::ParseBoolError),
    #[error("missing an expected XML attribute")]
    MissingAttr,
    #[error("missing data that should have already been parsed")]
    MissingData,
}
