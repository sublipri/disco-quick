# Disco Quick

Disco Quick is a library for processing the [Discogs](https://www.discogs.com) monthly [data dumps](http://www.discogs.com/data/) via iterators of structs. It uses [quick-xml's](https://github.com/tafia/quick-xml) streaming API with a state machine that was decoupled from [discogs-load](https://github.com/DylanBartels/discogs-load) and expanded to handle all the data in the dumps.

## Example:

```rust
use disco_quick::{DiscogsReader, DiscogsReader::*};

for arg in std::env::args().skip(1) {
    match DiscogsReader::from_path(&arg) {
        Ok(Artists(artists)) => artists.take(1).for_each(|a| println!("{a:#?}")),
        Ok(Labels(labels)) => labels.take(1).for_each(|l| println!("{l:#?}")),
        Ok(Masters(masters)) => masters.take(1).for_each(|m| println!("{m:#?}")),
        Ok(Releases(releases)) => releases.take(1).for_each(|r| println!("{r:#?}")),
        Err(e) => eprintln!("Error reading {arg}: {e}"),
    };
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
