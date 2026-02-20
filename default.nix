{
  rustPlatform,
  # native dependencies
  libpq,
  pkg-config,
  rust-bin,
  protobuf,
  # build dependencies
  jdk17,
  openssl
}:
rustPlatform.buildRustPackage rec {
  pname = "meesign-server";
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
  src = ./.;
  cargoBuildFlags = "--package ${pname}";

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "meesign-crypto-0.5.0" = "sha256-/ugNNNBnZTC58tTsXMbvwalMQh9sGgDkIFjUD+72T1A=";
    };
  };

  nativeBuildInputs = [
    libpq
    pkg-config
    rust-bin.beta.latest.default
    protobuf
  ];
  buildInputs = [
    jdk17
    openssl
  ];
}
