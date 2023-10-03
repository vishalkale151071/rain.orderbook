{
  description = "Flake for development workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rain_subgraph_cli.url = "github:rainprotocol/rain.subgraph-cli";
    flake-utils.url = "github:numtide/flake-utils";

  };

  outputs = {self, nixpkgs, rain_subgraph_cli, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rain-subgraph-cli = "${rain_subgraph_cli.defaultPackage.${system}}/bin/rain_subgraph";

      in rec {
        packages = rec {
          check-aver = ["Aver" "xd"];
          check-build =  pkgs.writeShellScriptBin "check-build" (''${rain-subgraph-cli} build'');



          # build-check= contract: ''
          #   ${(build-meta-cmd contract)} -o meta/${contract}.rain.meta;
          # '';
   

          check = pkgs.writeShellScriptBin "check" ("echo lol");

          default = check-build;


          # build-meta-cmd = contract: ''
          #   ${rain-cli} meta build \
          #     -i <(${rain-cli} meta solc artifact -c abi -i out/${contract}.sol/${contract}.json) -m solidity-abi-v2 -t json -e deflate -l en \
          #     -i src/concrete/${contract}.meta.json -m interpreter-caller-meta-v1 -t json -e deflate -l en \
          # '';
        };
      }
    );

}
