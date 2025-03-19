{
  description = "Development Environment with claude-code";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [ 
            claude-code
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
          ];
          shellHook = ''
            NIXPKGS_ALLOW_UNFREE=1
          '';
        };
      });
}
