{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay
    }:
    # The pinned nixpkgs no longer evaluates x86_64-darwin. The supported
    # development hosts are Apple Silicon and Linux, so keep this list
    # explicit instead of silently advertising the removed host platform.
    flake-utils.lib.eachSystem [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-linux"
    ] (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            rustToolchain
            rust-analyzer
            libiconv
            nushell
            llvmPackages.clang-unwrapped
            llvmPackages.lld
          ];

          shellHook = ''
            export CC=${llvmPackages.clang-unwrapped}/bin/clang
            export CXX=${llvmPackages.clang-unwrapped}/bin/clang++
          '';
        };
      }
    );
}
