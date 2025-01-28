use poise::serenity_prelude as serenity;
use sqlx::{
	FromRow,
	Row,
	sqlite::SqliteRow
};
use crate::utils::concat;

// CREATE TABLE IF NOT EXISTS settings
// (
// 	id						INT		PRIMARY KEY NOT NULL,
// 	serious				BOOL	NOT NULL DEFAULT true,
// 	messages			BOOL	NOT NULL DEFAULT false,
// 	roles					BOOL	NOT NULL DEFAULT false,
// 	roles_channel	INT		DEFAULT NULL
// );

#[derive(Debug)]
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
	pub group_message: serenity::ChannelId,
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
	pub role_id: serenity::RoleId,
}


// ========== OLD DATABASE STRUCTS!!!! ==========

/// Database feature entry
#[derive(FromRow, Debug)]
#[deprecated]
pub struct Server {
	/// ID of the server bitcast as an [i64].
	pub srvid: i64,
	/// Whether the server is serious or not, defaults to `true`.
	pub serious: bool,
	/// Whether the bot should respond to message events, defaults to `false`.
	pub messages: bool,
	/// Whether the bot should manage roles, defaults to `false`.
	pub roles: bool
}
impl Default for Server {
	fn default() -> Self {
		Self {
			srvid: 0,
			serious: true,
			messages: false,
			roles: false
		}
	}
}
// impl Into<serenity::GuildId> for Server {
// 	fn into(self) -> serenity::GuildId {
// 		serenity::GuildId::new(self.srvid as u64)
// 	}
// }
impl Server {
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

/// Database role feature settings entry.
#[derive(Default, FromRow, Debug)]
#[deprecated]
pub struct ServerRoles {
	/// ID of the server bitcast as an [i64].
	pub srvid: i64,
	/// ID of the server bitcast as an [i64].
	pub channel: Option<i64>
}

/// Database per-server role entry.
#[derive(Default, FromRow, Debug)]
#[deprecated]
pub struct RoleEntry {
	/// The ID of the role bitcast as an [i64].
	pub id: i64,
	/// [`String`] of the group that the role belongs to.
	pub grp: String,
	/// Roughly the amount of users with the role.
	pub users: u16
}
impl RoleEntry {
	/// Returns a [`serenity::RoleId`] from the id.
	pub fn extract_roleid (&self) -> serenity::RoleId {
		serenity::RoleId::new(self.id as u64)
	}
}

/// Database per-server group management.
#[derive(Default, FromRow, Debug)]
#[deprecated]
pub struct RoleGroup {
	/// [`String`] name of the role group.
	pub name: String,
	/// ID of the message that holds the group.
	pub msg: i64
}
