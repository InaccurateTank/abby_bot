use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use gumdrop::Options;
use tracing::instrument;

#[cfg(unix)]
fn to_pathbuf(s: &str) -> PathBuf {
	PathBuf::from(s)
}
#[cfg(windows)]
fn to_pathbuf(s: &str) -> PathBuf {
	PathBuf::from(s.replace("/", MAIN_SEPARATOR_STR))
}

#[derive(Options)]
pub struct Opts {
	pub help: bool,
	#[options(help = "Path to the directory where persistent data will be stored.", default = "data", meta = "<PATH>", parse(from_str = "to_pathbuf"))]
	pub data_dir: PathBuf,
	#[options(help = "The path to the bot configuration file. (default: <data_dir>/config.toml)", meta = "<PATH>", parse(from_str = "to_pathbuf"))]
	pub config: PathBuf,
	#[options(no_long, count, help = "Increase logging verbosity to DEBUG. Repeat once for TRACE data.")]
	pub verbose: u8
}

impl Opts {
	#[instrument]
	/// Thin wrapper around [parse_args_default_or_exit][gumdrop::Options::parse_args_default_or_exit()]
	pub fn parse() -> Self {
		let mut res = Opts::parse_args_default_or_exit();
		// data_dir is a directory, thus should end with a seperator
		if res.data_dir.ends_with(MAIN_SEPARATOR_STR) {
			res.data_dir.push(MAIN_SEPARATOR_STR);
		}
		// default configuration location
		if res.config.as_os_str().is_empty() {
			res.config = res.data_dir.join("config.toml");
		}
		res
	}
}
