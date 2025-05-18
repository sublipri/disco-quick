# Disco Quick

Disco Quick is a library for processing the [Discogs](https://www.discogs.com) monthly [data dumps](http://www.discogs.com/data/) via iterators of structs. It uses [quick-xml's](https://github.com/tafia/quick-xml) streaming API with a state machine that was decoupled from [discogs-load](https://github.com/DylanBartels/discogs-load) and expanded to handle all the data in the dumps.

## Example:

```rust
use disco_quick::DiscogsReader;
use std::env;

fn main() {
    for arg in env::args().skip(1) {
        let reader = match DiscogsReader::from_path(arg.as_ref()) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error reading {arg}. {e}");
                continue;
            }
        };
        match reader {
            DiscogsReader::Artists(artists) => {
                for artist in artists.take(100) {
                    println!("Artist ID {} is {}", artist.id, artist);
                }
            }
            DiscogsReader::Labels(labels) => {
                for label in labels.take(100) {
                    println!("Label ID {} is {}", label.id, label);
                }
            }
            DiscogsReader::Masters(masters) => {
                for master in masters.take(100) {
                    println!("Master ID {} is {}", master.id, master);
                }
            }
            DiscogsReader::Releases(releases) => {
                for release in releases.take(100) {
                    println!("Release ID {} is {}", release.id, release);
                }
            }
        };
    }
}
```

## Performance:

Running `examples/count.rs` with the 2023-10-01 dumps on a Ryzen 3900x with DDR4-3200 RAM produced the following results:

```bash
Parsed 8823813 artists in 20.476s (430914.2/s)
Parsed 2044219 labels in 03.186s (641475.25/s)
Parsed 2247215 masters in 23.626s (95113.26/s)
Parsed 16653652 releases in 12m30.206s (22198.756/s)
```

Peak memory usage was 5.2MB.
