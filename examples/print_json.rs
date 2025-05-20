use disco_quick::{ArtistsReader, DiscogsReader, LabelsReader, MastersReader, ReleasesReader};
use serde_json::to_string_pretty;
use std::env;

// cargo run --release --features serde --example print_json <PATH>...
fn main() {
    for arg in env::args().skip(1) {
        let reader = match DiscogsReader::from_path(&arg) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error reading {arg}. {e}");
                continue;
            }
        };
        match reader {
            DiscogsReader::Artists(artists) => handle_artists(*artists),
            DiscogsReader::Labels(labels) => handle_labels(*labels),
            DiscogsReader::Masters(masters) => handle_masters(*masters),
            DiscogsReader::Releases(releases) => handle_releases(*releases),
        };
    }
}

const AMOUNT: usize = 10;

fn handle_artists(artists: ArtistsReader) {
    for artist in artists.take(AMOUNT) {
        println!("{}", to_string_pretty(&artist).unwrap());
    }
}

fn handle_labels(labels: LabelsReader) {
    for label in labels.take(AMOUNT) {
        println!("{}", to_string_pretty(&label).unwrap());
    }
}

fn handle_masters(masters: MastersReader) {
    for master in masters.take(AMOUNT) {
        if master.artists.len() > 1 {
            println!("{}", to_string_pretty(&master).unwrap());
        }
    }
}

fn handle_releases(releases: ReleasesReader) {
    for release in releases.take(AMOUNT) {
        println!("{}", to_string_pretty(&release).unwrap());
    }
}
