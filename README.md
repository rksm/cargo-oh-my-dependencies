# `cargo omd` (cargo-oh-my-dependencies)

A cargo plugin to browse and edit crate features across a workspace.

This is still work in progress.

![screenshot](docs/screenshot.png)

## Usage

Run `cargo omd` to start the CLI.

Use the arrow keys to navigate the crates and features. Pressing `Enter` on a crate will create and open a graphviz visualization of the feature dependencies. Pressing `Enter` on a feature will toggle it on/off.

```
$ cargo omd --help

A cargo plugin to browse and edit crate features across a workspace.

Usage: cargo omd [MANIFEST]

Arguments:
  [MANIFEST]  Path to Cargo.toml file [default: Cargo.toml]

Options:
  -h, --help     Print help
  -V, --version  Print version
```
