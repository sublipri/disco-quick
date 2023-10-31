#![doc = include_str!("../README.md")]
pub mod artist;
pub mod artist_credit;
pub mod company;
pub mod label;
pub mod master;
mod parser;
pub mod reader;
pub mod release;
pub mod shared;
pub mod track;
mod util;
pub mod video;

pub use crate::reader::{
    ArtistsReader, DiscogsReader, LabelsReader, MastersReader, ReaderError, ReleasesReader,
};
