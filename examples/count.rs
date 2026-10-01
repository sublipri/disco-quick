use disco_quick::DiscogsReader::{self, *};
use std::time::{Duration, Instant};

/// Count the total items in a dump and report the parsing time.
fn main() {
    for arg in std::env::args().skip(1) {
        let reader = match DiscogsReader::from_path(&arg) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error reading {arg}. {e}");
                continue;
            }
        };
        let reader_name = reader.to_string();
        println!("Processing {arg}...");
        let now = Instant::now();
        let count = match reader {
            Releases(iter) => iter.count(),
            Masters(iter) => iter.count(),
            Artists(iter) => iter.count(),
            Labels(iter) => iter.count(),
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
        format!("{minutes:02}m{seconds:02}.{millis:03}s")
    } else {
        format!("{seconds:02}.{millis:03}s")
    }
}
