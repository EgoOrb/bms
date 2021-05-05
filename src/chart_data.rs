use crate::subsequence::*;
use encoding_rs::SHIFT_JIS;
use regex::Regex;
use std::{
    convert::TryFrom,
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

pub struct ChartData {
    pub player: u8,
    pub genre: String,
    pub title: String,
    pub artist: String,
    pub bpm: f64,
    pub play_level: u8,
    pub sound_paths: Vec<PathBuf>,
    pub subseqs: Vec<SubSequence>,
}

/// In Be-Music Source terminology, an object represents anything
/// that can appear in a chart.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Object {
    Silent,
    Note { sound: usize },
}

impl ChartData {
    fn from_data(data: String) -> Result<Self, Box<dyn Error>> {
        let r = Regex::new(r"#(.{3})(.{2}):(.*)").unwrap();
        let mut player = 0;
        let mut genre = "".to_string();
        let mut title = "".to_string();
        let mut artist = "".to_string();
        let mut bpm = 0.0;
        let mut level = 0;
        let mut paths = Vec::new();
        let mut subseqs = Vec::new();
        for line in data.lines() {
            if let Some((command, value)) = line.split_once(" ") {
                if !command.starts_with("#") {
                    continue;
                }

                match &command[1..] {
                    "PLAYER" => player = value.parse::<u8>()?,
                    "GENRE" => genre = value.to_string(),
                    "TITLE" => title = value.to_string(),
                    "ARTIST" => artist = value.to_string(),
                    "BPM" => bpm = value.parse::<f64>()?,
                    "PLAYLEVEL" => level = value.parse::<u8>()?,
                    _ => {
                        if command.starts_with("#WAV") || command.starts_with("#wav") {
                            paths.push(PathBuf::from(value));
                        }
                    }
                }
            }
        }

        for m in r.find_iter(data.as_str()) {
            if let Ok(seq) = SubSequence::try_from(m.as_str().to_string()) {
                subseqs.push(seq);
            }
        }

        Ok(ChartData {
            player,
            genre,
            title,
            artist,
            bpm,
            play_level: level,
            sound_paths: paths,
            subseqs,
        })
    }

    pub fn from_path(p: PathBuf) -> Result<Self, Box<dyn Error>> {
        let f = File::open(&p)?;
        let mut r = BufReader::new(f);

        let mut buffer: Vec<u8> = Vec::new();
        r.read(&mut buffer)?;

        // As most BMS charts are encoded as Shift-JIS, it's necessary
        // to convert the data to UTF-8 so Rust can work with it.
        let (data, _, invalid) = SHIFT_JIS.decode(&buffer);
        if !invalid {
            ChartData::from_data(data.into())
        } else {
            ChartData::from_data(String::from_utf8(buffer)?)
        }
    }

    pub fn count_measures(&self) -> usize {
        let mut subseqs2 = self.subseqs.clone();
        subseqs2.dedup_by_key(|x| x.measure);
        subseqs2.len()
    }

    pub fn get_measure(&self, measure: u32) -> Vec<SubSequence> {
        self.subseqs
            .iter()
            .filter(|&x| x.measure == measure)
            .map(|x| x.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_test() {
        let seq =
            SubSequence::try_from("#00111:0101010001".to_string()).expect("Failed to parse string");

        assert_eq!(seq.measure, 1);
        assert_eq!(seq.channel, 11);
        assert_eq!(
            seq.notes,
            vec![
                Object::Note { sound: 1 },
                Object::Note { sound: 1 },
                Object::Note { sound: 1 },
                Object::Silent,
                Object::Note { sound: 1 }
            ]
        );
    }
}
