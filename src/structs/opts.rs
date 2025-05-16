use std::path::PathBuf;
use gumdrop::Options;



#[cfg(not(target_os = "windows"))]
fn to_pathbuf(s: &str) -> PathBuf {
	use std::path::MAIN_SEPARATOR_STR;
	if !s.ends_with(MAIN_SEPARATOR_STR) {
		PathBuf::from(concat!(s, MAIN_SEPARATOR_STR))
	} else {
		PathBuf::from(s)
	}
}
#[cfg(target_os = "windows")]
fn to_pathbuf(s: &str) -> PathBuf {
	use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR};
	let mut path = s.replace("/", MAIN_SEPARATOR_STR);
	if !path.ends_with(MAIN_SEPARATOR) {
		path.push(MAIN_SEPARATOR);
	}
	PathBuf::from(path)
}

#[derive(Debug, Options)]
pub struct Opts {
	pub help: bool,
	#[options(help = "Path to the directory where persistent data will be stored.", default = "./data", meta = "<PATH>", parse(from_str = "to_pathbuf"))]
	pub data_dir: PathBuf,
	#[options(no_long, count, help = "Increase logging verbosity to DEBUG. Repeat once for TRACE data.")]
	pub verbose: u8
}

impl Opts {
	/// Extremely thin wrapper around [parse_args_default_or_exit][gumdrop::Options::parse_args_default_or_exit()]
	pub fn parse() -> Self {
		Opts::parse_args_default_or_exit()
	}
}
