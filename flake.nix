{
  description = "Kawaru - a stupidly simple text replacement tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        inherit (craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }) pname version;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          inherit pname version;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/${pname}";
        };
      }
    );
}
