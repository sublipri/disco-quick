use quick_xml::events::Event;
use thiserror::Error;

pub trait Parser {
    type Item;

    fn new() -> Self;

    fn take(&mut self) -> Self::Item;

    fn process(&mut self, ev: Event) -> Result<(), ParserError>;
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Bool(#[from] std::str::ParseBoolError),
}
