use poise::serenity_prelude as serenity;
use sqlx::FromRow;
use crate::concat;

// Server Features
/// Entry in the server manager
#[derive(FromRow, Debug)]
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
		Server {
			srvid: 0,
			serious: true,
			messages: false,
			roles: false
		}
	}
}
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

/// Table of per-server role settings. Currently just the channel the roles will be cast to.
#[derive(Default, FromRow, Debug)]
pub struct ServerRoles {
	/// ID of the server bitcast as an [i64].
	pub srvid: i64,
	/// ID of the server bitcast as an [i64].
	pub channel: Option<i64>
}

/// Individual entries for roles in the role list.
#[derive(Default, FromRow, Debug)]
pub struct RoleEntry {
	/// The ID of the role bitcast as an [i64].
	pub id: i64,
	/// [`String`] of the group that the role belongs to.
	pub grp: String,
	/// Roughly the amount of users with the role.
	pub users: u16
}

/// Seperate table of groups for [`RoleEntry`] structs.
#[derive(Default, FromRow, Debug)]
pub struct RoleGroup {
	/// [`String`] name of the role group.
	pub name: String,
	/// ID of the message that holds the group.
	pub msg: i64
}
