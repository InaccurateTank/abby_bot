-- Server Settings
CREATE TABLE guild_settings (
	guild_id			INT		PRIMARY KEY NOT NULL,
	admin					BOOL	NOT NULL DEFAULT false,
	unserious			BOOL	NOT NULL DEFAULT false,
	messages			BOOL	NOT NULL DEFAULT false,
	roles					BOOL	NOT NULL DEFAULT false,
	roles_channel	INT		DEFAULT	NULL
);

-- Role Groups
CREATE TABLE role_groups (
	guild_id		INT		NOT NULL,
  group_name	TEXT	NOT NULL,
  message_id	INT		NOT NULL,
	FOREIGN KEY (guild_id) REFERENCES guild_settings(guild_id)
		ON DELETE CASCADE,
	UNIQUE (guild_id, group_name)
);

-- Roles
CREATE TABLE roles (
	guild_id		INT		NOT NULL,
  group_name	TEXT 	NOT NULL,
  role_id			INT		NOT NULL,
	FOREIGN KEY (guild_id) REFERENCES guild_settings(guild_id)
		ON DELETE CASCADE,
	UNIQUE (guild_id, role_id)
);
CREATE TRIGGER no_orphaned_roles
	BEFORE INSERT ON roles
	WHEN NOT EXISTS (SELECT * FROM role_groups WHERE guild_id = NEW.guild_id AND group_name = NEW.group_name)
BEGIN
	SELECT RAISE(FAIL, 'No such role group exists yet.');
END;

SELECT
  l.roles_channel,
  r.message_id
FROM
  guild_settings l
  INNER JOIN role_groups r USING (guild_id)
WHERE guild_id = 2;
