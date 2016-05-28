# media_filename
A library for squeezing information out of filenames of torrents and media files.

## Usage

```rust
   extern crate media_filename;
    use media_filename::parse_filename;

    fn main() {
       let info = parse_filename("Super Awesome Series s02e03 2005 720p.mp4");

       println!("Title: {}", info.title.unwrap());
       println!("  Episode: {}, Season: {}", info.episode.unwrap(), info.season.unwrap());
       println!("  Year: {}", info.year.unwrap());
       println!("  Resolution: {}", info.resolution.unwrap());
       println!("  File extension: {}", info.extension.unwrap());
    }
```