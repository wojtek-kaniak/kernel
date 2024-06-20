{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
	nativeBuildInputs = with pkgs.buildPackages; [
		clang
		lld
		qemu
		powershell
		xorriso
        jq
        gdb
	];
}
