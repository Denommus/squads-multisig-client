{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      naersk,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };

        squads-multisig-client = pkgs.callPackage ./squads-multisig-client.nix { inherit naersk; };

        shell = pkgs.mkShell {
          inputsFrom = [ squads-multisig-client ];
        };
      in
      {
        packages = {
          inherit squads-multisig-client;
          default = squads-multisig-client;
        };

        devShells.default = shell;
      }
    );
}
