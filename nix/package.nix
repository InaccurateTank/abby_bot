{
  lib,
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  pname = "abby_bot";
  version = "0.5.0";

  nativeBuildInputs = [pkg-config];

  buildInputs = [openssl];

  src = ../.;

  cargoHash = "sha256-9wHNUHgHYnZsGx9t+LXYi+pQRQL0wDIAGnYbZ8693pY=";

  meta = {
    description = "A mediocre general purpose discord bot written in Rust.";
    homepage = "https://git.inaccuratetank.gay/inaccuratetank/abby_bot";
    license = lib.licenses.unlicense;
    mainProgram = "abby_bot";
  };
}
