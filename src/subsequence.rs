use std::convert::TryFrom;

use crate::chart_data::Object;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

/// (TODO: I'm not sure of the proper terminology, if there even is any)
/// A sequence of notes played at a given time.
#[derive(Clone)]
pub struct SubSequence {
    pub measure: u32,
    pub channel: u32,
    pub notes: Vec<Object>,
}

impl TryFrom<String> for SubSequence {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        lazy_static! {
            // Regex that matches strings with the format MMMCC:SS
            // where MMM represents the measure, CC represents the channel,
            // and SS represents the sound.
            static ref RE: Regex = Regex::new(r"#(.{3})(.{2}):(.*)").unwrap();
        }
        RE.captures(s.as_str())
            .and_then(|cap| {
                let measure = cap[1].parse::<u32>().unwrap_or(0);
                let channel = cap[2].parse::<u32>().unwrap_or(0);
                let notes = cap[3]
                    .to_string()
                    .chars()
                    .chunks(2)
                    .into_iter()
                    .map(|x| x.collect::<String>())
                    .map(|x| usize::from_str_radix(x.as_str(), 36))
                    .map_ok(|x| match x {
                        0 => Object::Silent,
                        _ => Object::Note { sound: x },
                    })
                    .filter_map(|x| x.ok())
                    .collect_vec();

                Some(SubSequence {
                    measure,
                    channel,
                    notes,
                })
            })
            .ok_or("Failed to parse sequence")
    }
}
