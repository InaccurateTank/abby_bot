use std::{fs, io, path::PathBuf, str::FromStr};
use color_eyre::{Report, Section};
use serde::Deserialize;
use tracing::{Level, info, instrument};
use crate::error::ConfigError;

#[derive(Deserialize)]
pub struct Config {
	token: Option<String>,
	token_file: Option<String>,
	#[serde(default = "LogConfig::default")]
	pub log: LogConfig
}
impl Config {
	pub fn load(
		path: impl AsRef<std::path::Path>
	) -> Result<Self, Report> {
		let config_path = path.as_ref();
		match fs::read_to_string(config_path) {
			Ok(file) => {
				let res = toml::from_str(&file)?;
				Ok(res)
			},
			Err(e) => {
				let generated: Result<bool, io::Error> = if e.kind() == io::ErrorKind::NotFound {
					if let Some(folder) = config_path.parent() {
						fs::create_dir_all(folder)?;
					}
					fs::write(config_path, include_str!("../../example_config.toml"))?;
					Ok(true)
				} else {
					Ok(false)
				};

				Err(ConfigError::MissingFile {
					path: config_path.into(),
					source: e
				}).with_suggestion(|| {
					if generated.is_ok_and(|x|x) {
						"Please fill out the generated config file before running."
					} else {
						"Please make sure the configuration path is either correct or writable before running."
					}
				})
			}
		}
	}

	#[instrument(skip_all)]
	pub fn read_token(&self) -> Result<String, Report> {
		if let Some(p) = &self.token_file {
			info!("Loading token from token file.");
			Ok(fs::read_to_string(p)?)
		} else if let Some(t) = &self.token {
			info!("Loading token from configuration.");
			Ok(t.to_owned())
		} else {
			Err(ConfigError::Token.into())
		}
	}
}

fn level_from_string<'de, D> (deserializer: D) -> std::result::Result<Level, D::Error>
where
	D: serde::Deserializer<'de>
{
	use serde::de;
	let string = String::deserialize(deserializer)?;
	Level::from_str(&string)
		.map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&string), &"a logging level"))
}

#[derive(Deserialize)]
#[serde(default)]
pub struct LogConfig {
	pub location: Option<PathBuf>,
	pub max_files: usize,
	#[serde(deserialize_with = "level_from_string")]
	pub level: tracing::Level
}
impl Default for LogConfig {
	fn default() -> Self {
		Self {
			location: None,
			max_files: 5,
			level: Level::INFO
		}
	}
}
