# abby_bot

abby_bot is a general purpose discord bot built as a hobby project. While originally starting out as purely an injoke creator, it has evolved into its current iteration of somthing resembling a general purpose discord bot. Also this was originally written in Javascript so that kind of sucked.

The general plan behind this bot is to integrate features that Discord has not or will not incorperate into their UI. So far these features include bulk message deletion and end-user role management. Any other features are frankly up to my whims on a moment to moment basis. Occasionally I'll sneak in a meme command. You've been warned.

## Configuration

The bot by default creates its needed files (A SQLite database and config file) in a data folder within its running directory. It can however be fed a both a persistent data folder and a configuration file. Choosing a data folder requires using the argument `--data_dir` or `-d` while configuration files take `--config` or `-c`. For example `abby_bot -c /etc/abby_bot/config.toml -d /var/lib/abbybot`.

As of this current moment the configuration file contains only the discord API token. This can be supplied with either the token itself or the location of a tokenfile.

## Installation

Currently only two methods are available.

### Nix

A flake with a module is provided for my personal convenience. Simply add the following to your Nix configuration. Please be aware that the token file will be visible to everyone within the nix store if stored within the flake. I recommend using [sops-nix](https://github.com/Mic92/sops-nix) for secret keeping.
```nix
# flake.nix
{
	inputs = {
		# ---Snip---
		abby_bot = {
			url = "git+https://git.inaccuratetank.gay/inaccuratetank/abby_bot"
			inputs.nixpkgs.follows = "nixpkgs";
		};
	}

	outputs = {nixpkgs, abby_bot, ...} @ inputs: {
		nixosConfigurations.HOSTNAME = nixpkgs.lib.nixosSystem {
			specialArgs = { inherit inputs; };
		}
	}
}
```
```nix
# configuration.nix
{
	inputs
}: {
	imports = [
		inputs.abby_bot.nixosModules.abby_bot
	];

	services.abby_bot = {
		enable = true;
		tokenFile = # Path to token file ;
	};
}
```

### Docker/Podman

A containerfile is also provided for use in a Docker or Podman setup. This is pinned to the latest stable version and can be run as-is. Simply bind a folder to `/abby_bot/data` to persist the database and configuration files.
