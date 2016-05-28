#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate itertools;

mod rope;

use rope::Rope;
use regex::Regex;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct MediaInfo {
    title:      Option<String>,
    group:      Option<String>,
    resolution: Option<String>,
    season:     Option<u32>,
    episode:    Option<u32>,
    source:     Option<String>,
    year:       Option<u32>,
    codec:      Option<String>,
    audio:      Option<String>,
    extension:  Option<String>
}

fn parse_pattern(rope: &mut Rope, regex: &Regex) -> Option<String> {
    let mut info: Option<(usize, String, (usize, usize))> = None;
    
    for (i, part) in rope.iter().enumerate() {
        if let Some(captures) = regex.captures(part.string) {
            let value = captures.iter().skip(1)
                .find(|x| x.is_some())
                .unwrap().unwrap().to_owned();
            let range = captures.pos(0).unwrap();
            
            info = Some((i, value, range));
            break;
        }
    };

    if let Some((i, value, (start, end))) = info {
        rope.mark_part_range(i, start..end);
        return Some(value);
    }

    None
}

pub fn parse_filename(filename: &str) -> MediaInfo {
    lazy_static! {
        static ref EXTENSION_REGEX: Regex = Regex::new("\\.([A-Za-z0-9]{2,4})$").unwrap();
        static ref RESOLUTION_REGEX: Regex = Regex::new("([0-9]{3,4}p)").unwrap();
        static ref GROUP_REGEX: Regex = Regex::new("(?:^\\[([^]]+)\\]|- ?([^-]+)$)").unwrap();
        static ref EPISODE_REGEX: Regex = Regex::new("(?:[eE]([0-9]{2,3})|[^0-9A-Za-z]([0-9]{2,3})(?:v[0-9])?[^0-9A-Za-z])").unwrap();
        static ref SEASON_REGEX: Regex = Regex::new("[sS]([0-9]{1,2})").unwrap();
        static ref SOURCE_REGEX: Regex = Regex::new("((?i)(?:PPV.)?[HP]DTV|(?:HD)?CAM|BRRIP|TS|(?:PPV )?WEB.?DL(?: DVDRip)?|HDRip|DVDRip|CamRip|W[EB]BRip|BluRay|DvDScr|hdtv)").unwrap();
        static ref YEAR_REGEX: Regex = Regex::new("((19[0-9]|20[01])[0-9])").unwrap();
        static ref CODEC_REGEX: Regex = Regex::new("((?i)xvid|x264|h\\.?264)").unwrap();
        static ref AUDIO_REGEX: Regex = Regex::new("((?i)MP3|DD5\\.?1|Dual[- ]Audio|LiNE|DTS|AAC(?:\\.?2\\.0)?|AC3(?:\\.5\\.1)?)").unwrap();
    }

    let mut rope = Rope::new(&filename);

    MediaInfo {
        extension:  parse_pattern(&mut rope, &EXTENSION_REGEX),
        source:     parse_pattern(&mut rope, &SOURCE_REGEX),
        codec:      parse_pattern(&mut rope, &CODEC_REGEX),
        audio:      parse_pattern(&mut rope, &AUDIO_REGEX),
        resolution: parse_pattern(&mut rope, &RESOLUTION_REGEX),
        group:      parse_pattern(&mut rope, &GROUP_REGEX),
        season:     parse_pattern(&mut rope, &SEASON_REGEX).and_then(|s| s.parse().ok()),
        year:       parse_pattern(&mut rope, &YEAR_REGEX).and_then(|s| s.parse().ok()),
        episode:    parse_pattern(&mut rope, &EPISODE_REGEX).and_then(|s| s.parse().ok()),

        title: {
            let x: &[_] = &['(', ')', '[', ']', ' ', '-', '_', '.'];

            let title = rope.unmarked()
                .map(|p| p.string)
                .sorted_by(|a,b| Ord::cmp(&a.len(), &b.len()))
                .pop();

            let title = title.map(|s| s.trim_matches(x));

            title.map(|s| s.to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_filename;

    macro_rules! assert_parse {
        ( $str:expr, {
            $($field:ident : $value:expr),*
        } ) => {
            {
                let info = parse_filename($str);
                println!("{:?}", info);
                $(assert!(info.$field == Some(($value).into()));)*
            }
        }
    }
    
    #[test]
    fn parse() {
        assert_parse!("[HorribleSubs] Mayoiga - 03 [720p].mkv", {
            group: "HorribleSubs",
            episode: 03 as u32,
            resolution: "720p",
            title: "Mayoiga"
        });

        assert_parse!("Game of Thrones Season 6 S06E05 720p Web Dl x264 Mrlss", {
            title: "Game of Thrones Season 6",
            season: 06 as u32,
            episode: 05 as u32,
            resolution: "720p",
            source: "Web Dl",
            codec: "x264"
        });

        assert_parse!("The Ones Below 2015 HDRip XViD-ETRG", {
            title: "The Ones Below",
            source: "HDRip",
            group: "ETRG",
            year: 2015 as u32,
            source: "HDRip",
            codec: "XViD"
        });
    }
}
