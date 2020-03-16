use lazy_static::lazy_static;
use anyhow::anyhow;
use regex::{Regex, RegexBuilder};
use ring::rand::{SystemRandom, SecureRandom};
use crate::errors::TerraResult;

lazy_static! {
    static ref RNG: SystemRandom = SystemRandom::new();
    static ref WHITESPACE_REGEX: Regex = RegexBuilder::new(r"\s+").build().unwrap();
}

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
        write!(&mut output, "{:X}", byte).unwrap();
    }
    output
}

pub fn generate_session_key() -> TerraResult<String> {
    let mut key = [0u8; 32];
    RNG.fill(&mut key).map_err(|_| anyhow!("unable to generate session key"))?;
    Ok(hexstring(&key))
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
