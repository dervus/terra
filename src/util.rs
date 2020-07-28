use std::{fs::File, io::BufReader, path::Path};
use log::info;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use serde::de::DeserializeOwned;

static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| RegexBuilder::new(r"\s+").build().unwrap());

pub fn capitalize<T: AsRef<str>>(input: T) -> String {
    let mut output = String::with_capacity(input.as_ref().len());
    for (index, character) in input.as_ref().to_owned().chars().enumerate() {
        if index == 0 {
            for upcase in character.to_uppercase() {
                output.push(upcase);
            }
        } else {
            output.push(character);
        }
    }
    output
}

pub fn hexstring<T: AsRef<[u8]>>(input: T) -> String {
    use std::fmt::Write;
    let input = input.as_ref();
    let mut output = String::with_capacity(input.len() * 2);
    for byte in input.iter() {
        write!(&mut output, "{:02X}", byte).unwrap();
    }
    output
}

pub fn prepare_name(input: &str) -> String {
    capitalize(WHITESPACE_REGEX.replace(&input.trim().to_lowercase(), " "))
}

pub fn prepare_name_extra(input: Option<&str>) -> Option<String> {
    input.and_then(|s| {
        let result = WHITESPACE_REGEX.replace(s.trim(), " ");
        if result.is_empty() {
            None
        } else {
            Some(result.into())
        }
    })
}

pub fn name_to_id(input: &str) -> String {
    input
        .trim()
        .replace(char::is_whitespace, "_")
        .replace(|c: char| (c != '_' && !c.is_alphanumeric()), "")
        .to_lowercase()
}

pub fn load_yaml<T, P>(path: P) -> anyhow::Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    info!("Loading file {:?}", path.as_ref());
    let file = File::open(path.as_ref())?;
    let yaml: T = serde_yaml::from_reader(BufReader::new(file))?;
    Ok(yaml)
}

pub fn load_markdown<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    info!("Loading file {:?}", path.as_ref());
    let source = std::fs::read_to_string(path.as_ref())?;
    let options = comrak::ComrakOptions {
        smart: true,
        ext_strikethrough: true,
        ext_table: true,
        ext_autolink: true,
        ext_tasklist: true,
        ext_superscript: true,
        ext_footnotes: true,
        ..Default::default()
    };
    Ok(comrak::markdown_to_html(&source, &options))
}
