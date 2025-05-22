//! Templating submodule that contains the constructors for status messages of various kinds.

use poise::serenity_prelude as serenity;
use crate::{colors, concat};

fn template_title (
	icon: &str,
	title: &str
) -> String {
	concat!(icon, " ", title, " ", icon)
}

pub fn error(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Error");
	serenity::CreateEmbed::default()
		.title(template_title(":exclamation:", title))
		.description(description)
		.color(colors::ERROR)
}

pub fn info(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Info");
	serenity::CreateEmbed::default()
		.title(template_title(":information_source:", title))
		.description(description)
		.color(colors::INFO)
}

pub fn processing() -> serenity::CreateEmbed {
	serenity::CreateEmbed::default()
		.title(template_title(":gear:", "Processing"))
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
		.title(template_title(":white_check_mark:", title))
		.description(description)
		.color(colors::INFO)
}

pub fn warning(
	title: Option<&str>,
	description: impl Into<String>
) -> serenity::CreateEmbed {
	let title = title.unwrap_or("Warning");
	serenity::CreateEmbed::default()
		.title(template_title(":warning:", title))
		.description(description)
		.color(colors::WARN)
}
