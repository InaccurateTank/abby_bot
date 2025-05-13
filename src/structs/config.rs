use color_eyre::Result;
use crate::error::{UserError, BotError};

const DEFAULT_CONFIG: &str = r##"# Abbybot config file
token = "INSERT_TOKEN""##;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
	pub token: String,
}
impl Config {
	pub fn load(
		path: impl AsRef<std::path::Path>
	) -> Result<Self> {
		match std::fs::read_to_string(path.as_ref().join("config.toml")) {
			// File exists
			Ok(file) => {
				// Return config file or deserializing error
				Ok(toml::from_str(&file)?)
			},
			// File doesn't exist
			Err(e) => {
				match e.kind() {
					// File does not exist
					std::io::ErrorKind::NotFound => {
						std::fs::write(path, DEFAULT_CONFIG)?;
						return Err(UserError(BotError::ConfigFileMissing.into()).into())
					},
					// Some other issue occured
					_ => Err(UserError(e.into()).into())
				}
			}
		}
		// Ok(toml::from_str(&config_file?)?)
		// let mut config_file = OpenOptions::new()
		// 	.create(true)
		// 	.truncate(false)
		// 	.read(true)
		// 	.write(true)
		// 	.open(path.as_ref().join("config.toml"))?;
		// let mut toml = String::new();
		// config_file.read_to_string(&mut toml)?;
		// if toml.is_empty() {
			// config_file.write_all(DEFAULT_CONFIG.as_bytes())?;
	// 		Err(UserError(BotError::ConfigFileMissing.into()).into())
	// 	} else {
	// 		Ok(toml::from_str(&toml)?)
	// 	}
	}
}
