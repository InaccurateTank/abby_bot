use color_eyre::Result;
use crate::Context;
use crate::utils;

fn byte_to_emoji(value: u8) -> String {
  let mut buffer = String::new();
  let mut value = value;
  if value == 0 {
    buffer = utils::concat(&buffer, "❤️");
  }
  loop {
    let (emoji, subtract) = match value {
      200..=255 => ("🫂", 200),
      50..=199 => ("💖", 50),
      10..=49 => ("✨", 10),
      5..=9 => ("🥺", 5),
      1..=4 => (",", 1),
      0 => break,
    };
    buffer = utils::concat(&buffer, emoji);
    value -= subtract;
  }
  buffer = utils::concat(&buffer, "👉👈");
  buffer
}

/// Use your words I don't speak bottom.
///
/// Listen this is literally just a bytecode translator. That's it. I don't know what to tell ya.
#[poise::command(
	check = "checks::serious",
  slash_command,
  category = "Misc"
)]
pub async fn bottomify(
  ctx: Context<'_>,
  #[description = "Text to translate"]
  plead: String
) -> Result<()> {
  let result = plead.bytes().map(byte_to_emoji).collect::<String>();
	ctx.send(poise::CreateReply::default()
		.content(result)
	).await?;
  Ok(())
}
