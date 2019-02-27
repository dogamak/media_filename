#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate itertools;

mod rope;

use rope::Rope;
use regex::Regex;
use itertools::Itertools;
use std::path::{Path, Component};

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title:      Option<String>,
    pub group:      Option<String>,
    pub resolution: Option<String>,
    pub season:     Option<u32>,
    pub episode:    Option<u32>,
    pub source:     Option<String>,
    pub year:       Option<u32>,
    pub codec:      Option<String>,
    pub audio:      Option<String>,
    pub extension:  Option<String>,
    pub checksum:   Option<String>,
    pub scene:      Option<String>,
    pub subs:       Option<String>,
    pub region:     Option<String>,
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

pub fn parse_path<P: AsRef<Path>>(path: P) -> MediaInfo {
    let mut rope = Rope::empty();

    for component in path.as_ref().components() {
        match component {
            Component::Normal(part) => {
                if let Some(s) = part.to_str() {
                    rope.append(s);
                }
            },
            _ => {},
        }
    }
    parse_rope(rope)
}

pub fn parse_filename<S: AsRef<str>>(filename: S) -> MediaInfo {
    parse_rope(Rope::new(filename.as_ref()))
}

fn parse_rope(mut rope: Rope) -> MediaInfo {
    lazy_static! {
        static ref EXTENSION_REGEX: Regex = Regex::new("\\.([A-Za-z0-9]{2,4})$").unwrap();
        static ref RESOLUTION_REGEX: Regex = Regex::new("([0-9]{3,4}p|[0-9]{3,4}x[0-9]{3,4})").unwrap();
        static ref GROUP_REGEX: Regex = Regex::new("(?:^\\[([^]]+)\\]|- ?([^-]+)$)").unwrap();
        static ref EPISODE_REGEX: Regex = Regex::new("(?:[eE]([0-9]{2,3})|[^0-9A-Za-z]([0-9]{2,3})(?:v[0-9])?[^0-9A-Za-z])").unwrap();
        static ref SEASON_REGEX: Regex = Regex::new("(?i:s|season.)([0-9]{1,2})").unwrap();
        static ref SOURCE_REGEX: Regex = Regex::new("((?i)(?:PPV.)?[HP]DTV|(?:HD)?CAM|BRRIP|[^a-z]TS[^a-z]|(?:PPV )?WEB.?DL(?: DVDRip)?|HDRip|DVDRip|CamRip|W[EB]BRip|BluRay|BD|DVD|DvDScr|hdtv)").unwrap();
        static ref YEAR_REGEX: Regex = Regex::new("((19[0-9]|20[01])[0-9])").unwrap();
        static ref CODEC_REGEX: Regex = Regex::new("((?i)xvid|x264|h\\.?264)").unwrap();
        static ref AUDIO_REGEX: Regex = Regex::new("((?i)MP3|DD5\\.?1|Dual[- ]Audio|LiNE|DTS|AAC(?:\\.?2\\.0)?|AC3(?:\\.5\\.1)?)").unwrap();
        static ref CRC_REGEX: Regex = Regex::new("\\[([0-9A-F]{8})\\]").unwrap();
        static ref SCNENE_TAG_REGEX: Regex = Regex::new("(PROPER|DIGITALLY|REMASTERED|RATED|UNRATED|FESTIVAL|LIMITED|INTERNAL|REPACK|EXTENDED|RECODE|RERIP|READNFO|STV|SE|DC|DL|FS|WS)").unwrap();
        static ref SUBS_REGEX: Regex = Regex::new("((?i)CUSTOM.SUBBED|SUBBED|UNSUBBED)").unwrap();
        static ref REGION_REGEX: Regex = Regex::new("((?i)R1|R2|R3|R4|R5|R6)").unwrap();
    }

    MediaInfo {
        extension:  parse_pattern(&mut rope, &EXTENSION_REGEX),
        checksum:   parse_pattern(&mut rope, &CRC_REGEX),
        source:     parse_pattern(&mut rope, &SOURCE_REGEX),
        codec:      parse_pattern(&mut rope, &CODEC_REGEX),
        audio:      parse_pattern(&mut rope, &AUDIO_REGEX),
        resolution: parse_pattern(&mut rope, &RESOLUTION_REGEX),
        group:      parse_pattern(&mut rope, &GROUP_REGEX),
        season:     parse_pattern(&mut rope, &SEASON_REGEX).and_then(|s| s.parse().ok()),
        year:       parse_pattern(&mut rope, &YEAR_REGEX).and_then(|s| s.parse().ok()),
        episode:    parse_pattern(&mut rope, &EPISODE_REGEX).and_then(|s| s.parse().ok()),
        scene:      parse_pattern(&mut rope, &SCNENE_TAG_REGEX),
        subs:       parse_pattern(&mut rope, &SUBS_REGEX),
        region:     parse_pattern(&mut rope, &REGION_REGEX),

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
    use super::*;

    macro_rules! assert_parse {
        ( $info:expr, {
            $($field:ident : $value:expr),*
        } ) => {
            {
                let info = $info;
                println!("{:?}", info);
                $(assert!(info.$field == Some(($value).into()));)*
            }
        }
    }

    #[test]
    fn test_parse_filename() {
        assert_parse!(parse_filename("[HorribleSubs] Mayoiga - 03 [720p].mkv"), {
            group: "HorribleSubs",
            episode: 03 as u32,
            resolution: "720p",
            title: "Mayoiga"
        });

        assert_parse!(parse_filename("Game of Thrones Season 6 S06E05 720p Web Dl x264 Mrlss"), {
            title: "Game of Thrones",
            season: 06 as u32,
            episode: 05 as u32,
            resolution: "720p",
            source: "Web Dl",
            codec: "x264"
        });

        assert_parse!(parse_filename("The Ones Below 2015 HDRip XViD-ETRG"), {
            title: "The Ones Below",
            source: "HDRip",
            group: "ETRG",
            year: 2015 as u32,
            source: "HDRip",
            codec: "XViD"
        });

        assert_parse!(parse_filename("Mega Movie (BD 1280x720 10bit)"), {
            title: "Mega Movie",
            source: "BD",
            resolution: "1280x720"
        });

        assert_parse!(parse_filename("[RightShiftBy2] Akagami no Shirayuki-hime - 15 [720p][6860573F].mp4"), {
            title: "Akagami no Shirayuki-hime",
            group: "RightShiftBy2",
            episode: 15 as u32,
            resolution: "720p",
            checksum: "6860573F",
            extension: "mp4"
        });
    }

    #[test]
    fn test_parse_path() {
        assert_parse!(parse_path(Path::new("Season 1/Mr. Robot - e03 - Episode Title.mp4")), {
            title: "Mr. Robot",
            season: 1 as u32,
            episode: 3 as u32,
            extension: "mp4"
        });
    }
}
