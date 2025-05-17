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

  cargoHash = "sha256-NBOfqlT5l91e9i+2kdEI2Uj1/cBnY1LTTDflDfZZ3+0=";

  meta = {
    description = "A mediocre general purpose discord bot written in Rust.";
    homepage = "https://git.inaccuratetank.gay/inaccuratetank/abby_bot";
    license = lib.licenses.unlicense;
    mainProgram = "abby_bot";
  };
}
