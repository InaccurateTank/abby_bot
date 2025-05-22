{
  description = "A mediocre general purpose discord bot written in Rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  } @ inputs: let
    forAllSystems = nixpkgs.lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux"
    ];
  in {
    packages = forAllSystems (system: {
      default = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix {};
      abby_bot = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix {};
    });

    nixosModules = {
      default = self.nixosModules.abby_bot;
      abby_bot = import ./nix/module.nix inputs;
    };
  };
}
