use std::{
	fs::OpenOptions,
	io::{Read, Write}
};
use color_eyre::{eyre::eyre, Result};
use serde::{Serialize, Deserialize};
use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub token: String,
}
impl Config {
	pub fn new(folder: &str) -> Result<Self> {
		let mut config_file = OpenOptions::new()
			.create(true)
			.truncate(false)
			.read(true)
			.write(true)
			.open(utils::concat(folder, "config.toml"))?;
		let mut toml = String::new();
		config_file.read_to_string(&mut toml)?;
		if toml.is_empty() {
			write!(config_file, r##"# Abbybot config file
token = "INSERT_TOKEN""##)?;
			Err(eyre!("Config file does not exist. Please fill out generated config file before running again."))
		} else {
			Ok(toml::from_str(&toml)?)
		}
	}
}
