{
  lib,
  rustPlatform,
  openssl,
  pkg-config
}:
rustPlatform.buildRustPackage {
  pname = "abby_bot";
  version = "1.0.0";

  nativeBuildInputs = [pkg-config];

  buildInputs = [openssl];

	buildFeatures = ["systemd"];

  src = ../.;

  cargoHash = "sha256-A2xkNKuCizeA1LKGvQ5U1Hot9bEfXb0Fp4Mt4sSQ64Q=";

  meta = {
    description = "A mediocre general purpose discord bot written in Rust.";
    homepage = "https://git.inaccuratetank.gay/inaccuratetank/abby_bot";
    license = lib.licenses.unlicense;
    mainProgram = "abby_bot";
  };
}
