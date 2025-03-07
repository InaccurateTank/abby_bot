use std::borrow::Cow;
use color_eyre::{Report, Result};
use poise::{serenity_prelude as serenity, Modal};
use crate::{
	Data,
	templates,
	error::{BotError, UserError}
};

fn id_split(url: String) -> Result<[u64;2]> {
	// Need to check if everything is in the correct format first
	let id_vec = url.split('/').collect::<Vec<&str>>();
	let [chstr, msgstr] = id_vec.as_slice()[id_vec.len()-2..] else {
		// return Err(error::Error::MalformedInput(url).into())
		return Err(UserError(BotError::MalformedInput(url).into()).into())
	};
	// Start Parsing
	let Ok(channel_result) = chstr.parse::<u64>() else {
		// return Err(error::Error::MalformedInput(url).into())
		return Err(UserError(BotError::MalformedInput(url).into()).into())
	};
	let Ok(message_result) = msgstr.parse::<u64>() else {
		// return Err(error::Error::MalformedInput(url).into())
		return Err(UserError(BotError::MalformedInput(url).into()).into())
	};
	// Return
	Ok([channel_result, message_result])
}

#[derive(Debug, poise::Modal)]
#[name = "Continue with mass delete?"]
struct ConfirmModal {
	#[placeholder = "Insert the final number in the from URL to continue"]
	confirm: String
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
	ctx: poise::ApplicationContext<'_, Data, Report>,
	#[description = "Ident of the message to start deleting from."]
	from: String,
	#[description = "Ident of the message to stop deletions at."]
	until: Option<String>
) -> Result<()> {
	// Parse from values
	let (from_channel, from_id) = id_split(from)
		.map(|[c, m]| (serenity::ChannelId::new(c), serenity::MessageId::new(m)))?;
	// let (from_channel, from_id) = match id_split(from) {
	// 	Ok([chid, msgid]) => (serenity::ChannelId::new(chid), serenity::MessageId::new(msgid)),
	// 	Err(err) => {
	// 		// Input failure
	// 		ctx.send(poise::CreateReply::default()
	// 			.ephemeral(true)
	// 			.embed(templates::state_embed(false, &format!("from: {}", err)))
	// 		).await?;
	// 		return Ok(())
	// 	}
	// };

	// Confirm
	let conf = ConfirmModal::execute(ctx).await?
		.and_then(|f| {
			if f.confirm != from_id.to_string() {
				return None
			}
			Some(f)
		}).is_none();
	if conf {
		// Confirm failure
		// ctx.send(poise::CreateReply::default()
		// 	.ephemeral(true)
		// 	.embed(templates::state_embed(false, "Confirmation failed, aborting."))
		// ).await?;
		// return Ok(());
		return Err(UserError(BotError::UnconfirmedModal.into()).into())
	}

	// Fetch messages, including the message selected.
	let mut messages = match from_channel.messages(ctx, serenity::GetMessages::new().after(from_id)).await {
		Ok(value) => value,
		Err(_) => {
			// from message Id is bad
			// ctx.send(poise::CreateReply::default()
			// 	.ephemeral(true)
			// 	.embed(templates::state_embed(false, "Input `from` is parseable but isn't valid. Either the id was entered incorrectly or doesn't exist in this server."))
			// ).await?;
			return Err(UserError(BotError::InvalidInput("from".to_string()).into()).into())
		}
	};
	messages.push(ctx.http().get_message(from_channel, from_id).await?);

	if let Some(until_value) = until {
		let (until_channel, until_id) = id_split(until_value)
			.map(|[c, m]| (serenity::ChannelId::new(c), serenity::MessageId::new(m)))?;
		// let [until_channel, until_id] = match id_split(until_value) {
		// 	Ok([chid, msgid]) => [chid, msgid],
		// 	Err(err_box) => {
		// 		// Input failure
		// 		ctx.send(poise::CreateReply::default()
		// 			.ephemeral(true)
		// 			.embed(templates::state_embed(false, &format!("until: {}", err_box)))
		// 		).await?;
		// 		return Ok(())
		// 	}
		// };
		// If they arn't from the same channel
		if from_channel != until_channel {
			// ctx.send(poise::CreateReply::default()
			// 	.ephemeral(true)
			// 	.embed(templates::state_embed(false, "Inputs are not in the same channel as each other. Aborting."))
			// ).await?;
			// return Ok(())
			return Err(UserError(BotError::InputChannelMismatch.into()).into())
		}
		// Find the position of the until message
		let position = match messages.iter().position(|f| {
			f.id == until_id
		}) {
			Some(v) => v,
			None => {
				// until message id is bad
				// ctx.send(poise::CreateReply::default()
				// 	.ephemeral(true)
				// 	.embed(templates::state_embed(false, "until: Input is parseable but isn't valid. Either the id was entered incorrectly or doesn't exist in this server."))
				// ).await?;
				// return Ok(());
				return Err(UserError(BotError::InvalidInput("until".to_string()).into()).into())
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

	// Delete and Display
	from_channel.delete_messages(ctx.serenity_context(), &messages).await?;
	ctx.send(poise::CreateReply::default()
		.ephemeral(true)
		.attachment(serenity::CreateAttachment::bytes(archive, archive_name))
		.embed(
			templates::status::success(
				Some("Messages Deleted"),
				format!("Messages {} through {} successfully deleted. An archive of the deleted messages has been attached for moderation purposes.", &messages.first().unwrap().id, &messages.last().unwrap().id)
			)
		)
		// .embed(templates::state_embed(true, &format!("Messages {} through {} successfully deleted. An archive of the deleted messages has been attached for moderation purposes.", &messages.first().unwrap().id, &messages.last().unwrap().id)))
	).await?;
	Ok(())
}
