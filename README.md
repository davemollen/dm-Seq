## dm-Seq

A monophonic sequencer effect written in Rust.
The effect can be compiled to a [Mod devices](https://moddevices.com/) or lv2 plugin.

## Table of contents:

- [Mod devices installation](#Mod-devices-installation)
- [Other plugin format installation](#Other-plugin-format-installation)
  - [LV2 installation](#LV2-installation)

## Mod devices installation

You can find the plugin for the Mod Dwarf [here](./lv2/dm-Seq.lv2/).

To build the plugin for your MOD device see [these instructions](https://github.com/moddevices/mod-plugin-builder).

## Other plugin format installation

A prerequisite to install any plugin is to have Rust installed on your machine.
Follow [these instructions](https://www.rust-lang.org/tools/install) to install Rust.

Below you can find the additional instructions per plugin format. These instructions might not be complete. Please let me know if anything's missing.

### LV2 installation

Go into the lv2 directory and run the `cargo build --release` command.
Once finished, copy the compiled plugin from [/target/release](./lv2/target/release) into your plugin folder.
