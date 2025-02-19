use std::{
	fs::OpenOptions,
	io::{Read, Write}
};
use serde::{Serialize, Deserialize};
use crate::{
	utils,
	Error,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub token: String,
}
impl Config {
	pub fn new(folder: &str) -> Result<Self, Error> {
		let mut file = OpenOptions::new()
			.create(true)
			.truncate(false)
			.read(true)
			.write(true)
			.open(utils::concat(folder, "config.toml"))?;
		let mut toml = String::new();
		file.read_to_string(&mut toml)?;
		if toml.is_empty() {
			write!(file, r##"# Abbybot config file
token = "INSERT_TOKEN""##)?;
			Err(String::from("Config file does not exist. Please fill out generated config file before running again.").into())
		} else {
			Ok(toml::from_str(&toml)?)
		}
	}
}
