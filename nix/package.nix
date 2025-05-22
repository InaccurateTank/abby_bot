{
  lib,
  rustPlatform,
  openssl,
  pkg-config
}:
rustPlatform.buildRustPackage {
  pname = "abby_bot";
  version = "0.5.0";

  nativeBuildInputs = [pkg-config];

  buildInputs = [openssl];

	buildFeatures = ["systemd"];

  src = ../.;

  cargoHash = "sha256-LTp+HUqBW8k6x0I6Y97oSJQr8AKbYVY9P0qqUM0ZaBU=";

  meta = {
    description = "A mediocre general purpose discord bot written in Rust.";
    homepage = "https://git.inaccuratetank.gay/inaccuratetank/abby_bot";
    license = lib.licenses.unlicense;
    mainProgram = "abby_bot";
  };
}
