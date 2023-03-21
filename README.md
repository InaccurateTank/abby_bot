# abby_bot

abby_bot is a general purpose discord bot built as a hobby project. While originally starting out as purely an injoke creator it has evolved into its current iteration, that being something that actually resembles an actual general purpose bot. Also this was originally written in Javascript so that kinda sucked.

The bot by default creates its needed files (A SQLite database and config file) in data folder within its running directory. It can however be fed a folder to store/read these files in using the argument `--data`. For example, `--data /etc/abbybot`. The config file in particular is required due to it containing the Discord REST API token which is required to run the bot in the first place.

## Docker

A containerfile is also provided for use in Docker or Podman. This is pinned to the latest stable version and can simply be run as-is. Simply bind a folder to `/abby_bot/data` to save database and config files.
