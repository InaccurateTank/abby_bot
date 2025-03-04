use std::{
	fs::OpenOptions,
	io::{Read, Write}
};
use color_eyre::Result;
use crate::{
	error::{UserError, BotError},
	utils
};

const DEFAULT_CONFIG: &str = r##"# Abbybot config file
token = "INSERT_TOKEN""##;

#[derive(Debug, serde::Deserialize)]
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
			config_file.write(DEFAULT_CONFIG.as_bytes())?;
			Err(UserError(BotError::ConfigFileMissing.into()).into())
		} else {
			Ok(toml::from_str(&toml)?)
		}
	}
}
