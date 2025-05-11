inputs: {
  config,
  lib,
  pkgs,
	...
}: let
  inherit (lib.options) mkOption mkEnableOption mkPackageOption;
	inherit (lib.modules) mkIf;
	inherit (pkgs.stdenv.hostPlatform) system;

  pkg = inputs.self.packages.${system}.abby_bot;
  cfg = config.programs.abby_bot;
in {
  options.programs.abby_bot = {
    enable = mkEnableOption "abby_bot systemd service.";

    package = mkPackageOption pkgs "abby_bot" {
      default = pkg;
    };

		dataFolder = mkOption {
			description = "Path of the folder used for config and database placement.";
			type = lib.type.path;
			default = /etc/abbybot;
		};
  };

  config = mkIf cfg.enable {
		# TODO: Fully declarative configuration

		systemd.services."abby_bot" = {
			description = "Discord bot systemd service.";
			wantedBy = [ "multi-user.target" ];
			serviceConfig = {
				Type = "simple";
				Restart = "on-failure";
				RestartSec = 10;

				MemoryDenyWriteExecute = mkDefault true;

				NoNewPrivileges = mkDefault true;
				LockPersonality = mkDefault true;
				RemoveIPC = mkDefault true;

				ProtectSystem = mkDefault "strict";
				ProtectHome = mkDefault true;

				PrivateTmp = mkDefault true;
				PrivateDevices = mkDefault true;
				ProtectHostname = mkDefault true;
				ProtectKernelTunables = mkDefault true;
				ProtectKernelModules = mkDefault true;
				ProtectControlGroups = mkDefault true;
				ProtectClock = mkDefault true;

				ProtectProc = mkDefault "invisible";
				ProcSubset = mkDefault "pid";

				RestrictNamespaces = mkDefault true;
				RestrictRealtime = mkDefault true;
				RestrictSUIDSGID = mkDefault true;

				ExecPaths = ["/nix/store"];
				NoExecPaths = ["/"];

				# ExecStartPre = ["${pkg}/bin/abby_bot -t -c ${configFile}"];
				ExecStart = "${pkg}/bin/abby_bot -d ${dataFolder}";
			};
		};
  };
}
