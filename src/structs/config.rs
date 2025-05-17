use std::{fs, io};
use color_eyre::{Report, Section};
use tracing::{info, instrument, warn};
use crate::error::ConfigError;

const DEFAULT_CONFIG: &str = r##"# Abbybot config file
token = "INSERT_TOKEN"
# Alternatively you can use a file with the token inside of it.
# This will be parsed before the token setting.
# token_file = "PATH_TO_TOKEN"##;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
	pub token: Option<String>,
	pub token_file: Option<String>
}
impl Config {
	#[instrument(fields(path = %path.as_ref().to_string_lossy()))]
	pub fn load(
		path: impl AsRef<std::path::Path>
	) -> Result<Self, Report> {
		let config_path = path.as_ref();
		match fs::read_to_string(&config_path) {
			Ok(file) => Ok(toml::from_str(&file)?),
			Err(e) => {
				let generated: Result<bool, io::Error> = if e.kind() == io::ErrorKind::NotFound {
					if let Some(folder) = config_path.parent() {
						fs::create_dir_all(folder)?;
					}
					fs::write(config_path, DEFAULT_CONFIG)?;
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

	#[instrument(skip(self))]
	pub fn read_token(&self) -> Result<String, Report> {
		if let Some(p) = &self.token_file {
			info!("Loading token from configuration.");
			Ok(fs::read_to_string(p)?)
		} else if let Some(t) = &self.token {
			info!("Loading token from token file.");
			Ok(t.to_owned())
		} else {
			Err(ConfigError::Token.into())
		}
	}
}
