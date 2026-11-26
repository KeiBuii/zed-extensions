# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A monorepo of [Zed](https://zed.dev) editor extensions, intended to be published as open source. Each extension is an independent unit in its own folder under `extensions/<extension-id>/`. There is no root build; every extension builds on its own.

## Extension layout

Each `extensions/<extension-id>/` must contain an `extension.toml` manifest (`id`, `name`, `version`, `schema_version = 1`, `authors`, `description`, `repository`). The folder name should match the `id`.

Two kinds of extensions:
- **Declarative only** (themes, icon themes, snippets, language configs using an existing grammar): just `extension.toml` plus `themes/`, `icon_themes/`, `snippets/`, `languages/<lang>/` files. No Rust.
- **With Rust code** (language servers, slash commands, debug adapters, context servers): add `Cargo.toml` with `crate-type = ["cdylib"]` and a `zed_extension_api` dependency, and `src/lib.rs` implementing `zed::Extension` registered via `zed::register_extension!`. Zed compiles it to WebAssembly (`wasm32-wasip2`) itself.

Tree-sitter grammars are referenced from `extension.toml` (`[grammars.<name>]` with `repository` + `rev`) and cloned by Zed into `grammars/` at build time; that folder is gitignored.

Extensions cannot add arbitrary UI or change core editor behavior; that requires forking Zed.

## Commands

For an extension with Rust code (run inside its folder):

```bash
cargo check --target wasm32-wasip2   # type-check (rustup target add wasm32-wasip2 first)
cargo clippy --target wasm32-wasip2
cargo test                            # host-side unit tests, if any
```

Running an extension: in Zed, command palette → `zed: install dev extension` → pick `extensions/<extension-id>`. Build errors and extension `println!`/`dbg!` output appear in `zed: open log`. Rust must be installed via rustup for Zed to build dev extensions.

## Publishing

Published by PR to `zed-industries/extensions`, which adds this repo as a git submodule and points each entry at its subfolder via `path = "extensions/<extension-id>"`. Bump `version` in the extension's `extension.toml` (and `Cargo.toml` if present) to release. Zed's registry requires an accepted open-source license file; the repo root `LICENSE` is MIT.

When adding an extension, also add a row to the Extensions table in `README.md`.

## Git

Commits are authored as Kei Bui (configured in the repo's local git config).
