use poise::serenity_prelude as serenity;
use sqlx::{
	FromRow,
	Row,
	sqlite::SqliteRow
};
use crate::{
	Error,
	utils::concat
};

// CREATE TABLE IF NOT EXISTS server_settings
// (
// 	id						INT		PRIMARY KEY NOT NULL,
// 	serious				BOOL	NOT NULL DEFAULT true,
// 	messages			BOOL	NOT NULL DEFAULT false,
// 	roles					BOOL	NOT NULL DEFAULT false,
// 	roles_channel	INT		DEFAULT NULL
// );

#[derive(Debug, Clone)]
pub struct ServerSettings {
	/// [`serenity::GuildId`] of the server, stored as an [i64].
	pub id: serenity::GuildId,
	/// Whether the server is serious or not, default `true`.
	pub serious: bool,
	/// Whether the bot should respond to message events, default `false`.
	pub messages: bool,
	/// Whether the bot should manage roles, default `false`.
	pub roles: bool,
	/// Channel to manage roles from if applicable, default `None`.
	pub roles_channel: Option<serenity::ChannelId>
}
impl ServerSettings {
	pub fn new(guild_id: serenity::GuildId) -> Self {
		let mut result = Self::default();
		result.id = guild_id;
		result
	}

	pub async fn from_query(guild_id: serenity::GuildId, db: &sqlx::SqlitePool) -> Result<Option<Self>, sqlx::Error> {
		sqlx::query_as("SELECT * FROM server_settings WHERE id = ?;")
			.bind(guild_id.get() as i64)
			.fetch_optional(db)
			.await
	}

	pub fn as_array(&self) -> [(&str, bool); 3] {
		[
			("serious", self.serious),
			("messages", self.messages),
			("roles", self.roles)
		]
	}

	pub fn as_selectmenuoptions(&self) -> Vec<serenity::CreateSelectMenuOption> {
		let mut opts = Vec::new();
		for (name, value) in self.as_array() {
			opts.push(serenity::CreateSelectMenuOption::new(name[0..1].to_uppercase() + &name[1..], concat("enable_", name))
				.default_selection(value)
				.to_owned());
		}
		opts
	}
}
impl Default for ServerSettings {
	fn default() -> Self {
		Self {
			id: serenity::GuildId::default(),
			serious: true,
			messages: false,
			roles: false,
			roles_channel: None
		}
	}
}
impl FromRow<'_, SqliteRow> for ServerSettings {
	fn from_row(row: &SqliteRow) -> sqlx::Result<Self, sqlx::Error> {

		let mapped_id = match row.try_get::<u64, &str>("id") {
			Ok(raw) => serenity::GuildId::from(raw),
			Err(e) => return Err(e)
		};

		let mapped_roles_channel = match row.try_get::<Option<u64>, &str>("roles_channel") {
			Ok(raw) => raw.map(|inner| serenity::ChannelId::from(inner)),
			Err(e) => return Err(e)
		};

		Ok(
			Self {
				id: mapped_id,
				serious: row.try_get::<bool, &str>("serious")?,
				messages: row.try_get::<bool, &str>("messages")?,
				roles: row.try_get::<bool, &str>("roles")?,
				roles_channel: mapped_roles_channel
			}
		)
	}
}

// CREATE TABLE IF NOT EXISTS role_groups(
// 	server_id			INT		NOT NULL,
//   group_name 		TEXT	NOT NULL,
//   group_message INT		NOT NULL,
// 	FOREIGN KEY (server_id) REFERENCES settings(id)
// 		ON DELETE CASCADE,
// 	UNIQUE (server_id, group_name)
// );
#[derive(FromRow, Debug)]
pub struct Group {
	#[sqlx(try_from = "u64")]
	/// [`serenity::GuildId`] of the server, stored as an [i64].
	pub server_id: serenity::GuildId,
	/// Name of the role group
	pub group_name: String,
	#[sqlx(try_from = "u64")]
	/// [`serenity::MessageId`] that the group is posted in, stored as an [`i64`].
	pub group_message: serenity::MessageId,
}


// CREATE TABLE IF NOT EXISTS roles (
// 	server_id		INT		NOT NULL,
//   group_name	TEXT 	NOT NULL,
//   role_id 		INT		NOT NULL,
// 	FOREIGN KEY (server_id) REFERENCES settings(id)
// 		ON DELETE CASCADE,
// 	UNIQUE (server_id, role_id)
// );
#[derive(FromRow, Debug)]
pub struct Role {
	#[sqlx(try_from = "u64")]
	/// [`serenity::GuildId`] of the server, stored as an [i64].
	pub server_id: serenity::GuildId,
	/// Name of the group that the role is under.
	pub group_name: String,
	#[sqlx(try_from = "u64")]
	/// [`serenity::RoleId`] of the role, stored as an [`i64`].
	pub role_id: serenity::RoleId
}
