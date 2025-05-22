inputs: {
  config,
  lib,
  pkgs
}: let
	inherit (lib) types;
  inherit (lib.options) mkOption mkEnableOption mkPackageOption;
	inherit (lib.modules) mkIf;
	inherit (pkgs.stdenv.hostPlatform) system;

  pkg = inputs.self.packages.${system}.abby_bot;
	cfg = config.services.abby_bot;
in {
  options.services.abby_bot = {
    enable = mkEnableOption "abby_bot systemd service.";

    package = mkPackageOption pkgs "abby_bot" {
      default = pkg;
    };

		dataDir = mkOption {
			description = "Path of the folder used for persistent data storage.";
			type = types.path;
			default = /var/lib/abby_bot;
		};

		tokenFile = mkOption {
			description = "The path to a file containing the Discord API token.";
			type = types.nullOr types.path;
			default = null;
		};
  };

  config = let
		format = pkgs.formats.toml {};
		configFile = format.generate "abby_bot.toml" {
			token_file = cfg.tokenFile;
		};
	in mkIf cfg.enable {
		users = {
			users.abby_bot = {
				group = "abby_bot";
				isSystemUser = true;
			};
			groups.abby_bot = {};
		};

		systemd.services."abby_bot" = {
			description = "AbbyBot general purpose Discord bot";

			after = [ "network-online.target" ];
			wants = [ "network-online.target" ];
			wantedBy = [ "multi-user.target" ];

			serviceConfig = {
				# Basic
				Type = "notify";
				User = "abby_bot";
				Group = "abby_bot";
				Restart = "on-failure";
				RestartSec = 10;

				UMask = "0027";
				# Capabilities
				CapabilityBoundingSet = "";
				AmbientCapabilities = "";
				# Security
				NoNewPrivileges = true;
				# Access write directories
				ReadWritePaths = [
					cfg.dataDir
				];
				# Sandboxing
				ProtectSystem = "strict";
				ProtectHome = true;
				PrivateTmp = true;
				PrivateDevices = true;
				DevicePolicy = "closed";
				ProtectProc = "invisible";
				RestrictAddressFamilies = [
					"AF_INET"
					"AF_INET6"
					"AF_UNIX"
				];
				LockPersonality = true;
				MemoryDenyWriteExecute = true;
				RestrictNamespaces = true;
				RestrictSUIDSGID = true;
				RestrictRealtime = true;
				RemoveIPC = true;
				# System Call Filtering
				SystemCallArchitectures = "native";

				ExecPaths = ["${pkg}"];
				NoExecPaths = ["/"];

				# ExecStartPre = ["${pkg}/bin/abby_bot -t -c ${configFile}"];
				ExecStart = "${pkg}/bin/abby_bot -c ${configFile} -d ${cfg.dataDir}";
			};
		};
  };
}
