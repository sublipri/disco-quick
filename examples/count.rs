use disco_quick::DiscogsReader;
use std::env;
use std::time::{Duration, Instant};

/// Count the total items in a dump and report the parsing time.
fn main() {
    for arg in env::args().skip(1) {
        let reader = match DiscogsReader::from_path(arg.as_ref()) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error reading {arg}. {e}");
                continue;
            }
        };
        let reader_name = reader.to_string();
        println!("Processing {}...", arg);
        let now = Instant::now();
        let count = match reader {
            DiscogsReader::Artists(artists) => artists.count(),
            DiscogsReader::Labels(labels) => labels.count(),
            DiscogsReader::Masters(masters) => masters.count(),
            DiscogsReader::Releases(releases) => releases.count(),
        };
        let duration = now.elapsed();
        let per_second = count as f32 / duration.as_secs_f32();
        println!(
            "Parsed {} {} in {} ({}/s)",
            count,
            reader_name,
            format_duration(duration),
            per_second
        );
    }
}

fn format_duration(d: Duration) -> String {
    let seconds = d.as_secs();
    let millis = d.subsec_millis();
    if seconds > 60 {
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        format!("{:02}m{:02}.{:03}s", minutes, seconds, millis)
    } else {
        format!("{:02}.{:03}s", seconds, millis)
    }
}
