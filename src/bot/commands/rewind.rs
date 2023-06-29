use std::borrow::Cow;
use poise::serenity_prelude::{self as serenity, CacheHttp};
use crate::{Context, Error, templates};

fn id_split(url: String) -> Result<[u64;2], Error> {
	// Need to check if everything is in the correct format first
	let id_vec = url.split('/').collect::<Vec<&str>>();
	let [chstr, msgstr] = id_vec.as_slice()[id_vec.len()-2..] else {
		return Err(Error::from("Input is malformed, please read the help for this command and try again."))
	};
	// Start Parsing
	let Ok(chres) = chstr.parse::<u64>() else {
		return Err(Error::from("Channel id isn't parseable. Please read the help for this command and try again."))
	};
	let Ok(msgres) = msgstr.parse::<u64>() else {
		return Err(Error::from("Message id isn't parseable. Please read the help for this command and try again."))
	};
	// Return
	Ok([chres, msgres])
}

/// Mass message removal tool. Please read the detailed help instructions before use.
///
/// Rewind is an administrative tool used to mass delete entire conversations within a channel.
/// It does this by parsing the URL of each input for the respective channel and message ids.
/// To get these URLs, right click the respective message(s) and select `Copy Message Link`.
/// The command will proceed to delete ***ALL*** messages since (and including) the "from" message.
/// If an "until" message is provided, the command will stop with deleting that message.
/// Due to the power and ability to accidentally use this, rewind requires confirmation before finalizing input.
/// Additionally the command must be used within the server it is meant for.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_MESSAGES",
	ephemeral
)]
pub async fn rewind(
	ctx: Context<'_>,
	#[description = "Ident of the message to start deleting from."]
	from: String,
	#[description = "Ident of the message to stop deletions at."]
	until: Option<String>
) -> Result<(), Error> {
	// Parse from values
	let [from_channel, from_id] = match id_split(from) {
		Ok([chid, msgid]) => [chid, msgid],
		Err(err_box) => {
			// Input failure
			ctx.send(|m| {
				m.content("");
				m.ephemeral(true);
				m.embed(|e| {
					templates::builder_state_embed(e, false, &format!("from: {}", err_box));
					e
				})
			}).await?;
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
			ctx.send(|m| {
				m.content("");
				m.ephemeral(true);
				m.embed(|e| {
					templates::builder_state_embed(e, false, "from: Input is parseable but isn't valid. Either the id was entered incorrectly or doesn't exist in this server.");
					e
				})
			}).await?;
			return Ok(());
		}
	};
	messages.push(ctx.http().get_message(from_channel, from_id).await?);

	if let Some(until_value) = until {
		let [until_channel, until_id] = match id_split(until_value) {
			Ok([chid, msgid]) => [chid, msgid],
			Err(err_box) => {
				// Input failure
				ctx.send(|m| {
					m.content("");
					m.ephemeral(true);
					m.embed(|e| {
						templates::builder_state_embed(e, false, &format!("until: {}", err_box));
						e
					})
				}).await?;
				return Ok(())
			}
		};
		// If they arn't from the same channel
		if from_channel != until_channel {
			ctx.send(|m| {
				m.content("");
				m.ephemeral(true);
				m.embed(|e| {
					templates::builder_state_embed(e, false, "Inputs are not in the same channel as each other. Aborting.");
					e
				})
			}).await?;
			return Ok(())
		}
		// Find the position of the until message
		let position = match messages.iter().position(|f| {
			f.id == until_id
		}) {
			Some(v) => v,
			None => {
				// until message id is bad
				ctx.send(|m| {
					m.content("");
					m.ephemeral(true);
					m.embed(|e| {
						templates::builder_state_embed(e, false, "until: Input is parseable but isn't valid. Either the id was entered incorrectly or doesn't exist in this server.");
						e
					})
				}).await?;
				return Ok(());
			}
		};
		// Use the position and delete all entries before it
		messages.drain(..position);
	}

	// Create Archive
	let archive = Cow::from(messages.iter()
		.rev()
		.flat_map(|f| {
			format!("{} - {}\n", f.author.name, f.content).into_bytes()
		}).collect::<Vec<u8>>());
	let archive_name = format!("Archive-{}-{}.txt", ctx.guild().unwrap().name, time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Iso8601::DEFAULT)?);

	// Delete and Confirm
	// serenity::ChannelId(from_channel).delete_messages(ctx, &messages).await?;
	ctx.send(|m| {
		m.content("");
		m.ephemeral(true);
		m.attachment(serenity::AttachmentType::Bytes { data: archive, filename: archive_name });
		m.embed(|e| {
			templates::builder_state_embed(e, true, &format!("Messages {} through {} successfully deleted. An archive of the deleted messages has been attached for moderation purposes.", &messages.first().unwrap().id, &messages.last().unwrap().id));
			e
		})
	}).await?;
	Ok(())
}
