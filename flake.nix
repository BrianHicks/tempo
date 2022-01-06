{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nmattia/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = naersk.lib."${system}";
      in rec {
        packages.tempo = naersk-lib.buildPackage {
          root = ./.;

          doCheck = true;
          checkPhase = "cargo test";
        };
        defaultPackage = packages.tempo;
        overlay = final: prev: { tempo = packages.tempo; };

        devShell = pkgs.mkShell {
          packages =
            [ pkgs.cargo pkgs.clippy pkgs.libiconv pkgs.rustc pkgs.rustfmt ];
        };
      });
}
