use std::ops::{Sub, Add};
use poise::serenity_prelude::{self as serenity, CacheHttp};
use crate::{Context, Error, templates};

fn id_split(combo: String) -> Option<(u64, u64)> {
	// Need to check if everything is in the correct format first
	match combo.split_once('/') {
		Some((chstr, msgstr)) => {
			let channel = match chstr.parse::<u64>() {
				Ok(chid) => chid,
				Err(_) => {
					// Parsing error
					return None
				}
			};
			let message = match msgstr.parse::<u64>() {
				Ok(msgid) => msgid,
				Err(_) => {
					// Parsing error
					return None
				}
			};
			// Correct output
			Some((channel, message))
		},
		// Parsing Error
		None => None
	}
}

#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	ephemeral
)]
pub async fn rewind(
	ctx: Context<'_>,
	from: String,
	until: Option<String>
) -> Result<(), Error> {
	// Parse from values
	let (from_channel, from_id) = match id_split(from) {
		Some((chid, msgid)) => (chid, msgid),
		None => {
			// Malformed input
			println!("Malformed Input");
			return Ok(())
		}
	};

	// Fetch messages, including the message selected.
	let mut messages = match serenity::ChannelId(from_channel)
		.messages(ctx, |retriever| {
			retriever.after(from_id)
		}).await {
		Ok(value) => value,
		Err(_) => {
			// from message Id is bad
			println!("from Id is bad");
			return Ok(());
		}
	};
	messages.push(ctx.http().get_message(from_channel, from_id).await?);

	if let Some(until_value) = until {
		let (until_channel, until_id) = match id_split(until_value) {
			Some((chid, msgid)) => (chid, msgid),
			None => {
				// Malformed input
				println!("Malformed Input");
				return Ok(())
			}
		};
		// If they arn't from the same channel
		if from_channel != until_channel {
			println!("Not same channel");
			return Ok(())
		}
		// Find the position of the until message
		let position = match messages.iter().position(|f| {
			f.id == until_id
		}) {
			Some(v) => v,
			None => {
				// until message id is bad
				println!("until Id is bad");
				return Ok(());
			}
		};
		// Use the position and delete all entries before it
		messages.drain(..position);
	}
	let msg: Vec<String> = messages.into_iter().map(|f| {
		format!("{} - {}", f.author.name, f.content)
	}).collect();
	println!("{msg:#?}");
	Ok(())
}
