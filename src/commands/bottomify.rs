use crate::{Context, Error};
use crate::{
	structs::db,
	utils
};

// fn concat(a: &str, b: &str) -> String {
//   let mut result: String = String::with_capacity(a.len() + b.len());
//   result += a;
//   result += b;
//   result
// }

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
  slash_command,
  category = "Misc"
)]
pub async fn bottomify(
  ctx: Context<'_>,
  #[description = "Text to translate"]
  plead: String
) -> Result<(), Error> {
	let srv_features = sqlx::query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(ctx.guild_id().unwrap().get() as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if srv_features.serious {
		utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}
  let result = plead.bytes().map(byte_to_emoji).collect::<String>();
	ctx.send(poise::CreateReply::default()
		.content(result)
	).await?;
  Ok(())
}
