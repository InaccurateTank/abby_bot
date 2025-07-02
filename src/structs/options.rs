use std::path::PathBuf;
use gumdrop::Options;
use tracing::instrument;

#[cfg(unix)]
fn to_pathbuf(s: &str) -> PathBuf {
	PathBuf::from(s)
}
#[cfg(windows)]
fn to_pathbuf(s: &str) -> PathBuf {
	use std::path::MAIN_SEPARATOR_STR;
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
		// default configuration location
		if res.config.components().count() == 0 {
			res.config = res.data_dir.join("config");
			res.config.set_extension("toml");
		}
		res
	}
}
