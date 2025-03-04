//! Templating submodule that contains the constructors for status messages of various kinds.

use poise::serenity_prelude as serenity;
use crate::colors;

pub fn error(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Error");
	serenity::CreateEmbed::default()
		.title(format!(":exclamation: {title} :exclamation:"))
		.description(description)
		.color(colors::ERROR)
}

pub fn info(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Info");
	serenity::CreateEmbed::default()
		.title(format!(":information_source: {title} :information_source:"))
		.description(description)
		.color(colors::INFO)
}

pub fn processing() -> serenity::CreateEmbed {
	serenity::CreateEmbed::default()
		.title(":gear: Processing :gear:")
		.description("Please wait...")
		.color(colors::WARN)
		.footer(serenity::CreateEmbedFooter::new("This will finish before the heat death of the universe, I promise."))
}

pub fn success(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Success");
	serenity::CreateEmbed::default()
		.title(format!(":white_check_mark: {title} :white_check_mark:"))
		.description(description)
		.color(colors::INFO)
}

pub fn warning(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Warning");
	serenity::CreateEmbed::default()
		.title(format!(":warning: {title} :warning:"))
		.description(description)
		.color(colors::WARN)
}
