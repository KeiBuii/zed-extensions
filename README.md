# Zed Extensions

A collection of extensions for the [Zed](https://zed.dev) editor, kept in one repository.
Each extension lives in its own folder under `extensions/`.

## Extensions

| Extension | Description |
|-----------|-------------|
| [htmx-language-server](extensions/htmx-language-server) | HTMX attribute completion (htmx-lsp) and snippets for HTML and template languages |

## Layout

```
extensions/
  <extension-id>/
    extension.toml   # manifest (required)
    Cargo.toml       # only for extensions with Rust code
    src/lib.rs       # only for extensions with Rust code
    themes/ languages/ snippets/ ...
```

## Developing locally

1. Install Rust via [rustup](https://rustup.rs) (Zed needs a rustup-managed toolchain to build Rust extensions).
2. In Zed, open the command palette and run `zed: install dev extension`.
3. Select the extension's folder, e.g. `extensions/<extension-id>`.

Use `zed: open log` to see build errors and `println!` / `dbg!` output from extensions.

## Publishing

Extensions are published by opening a PR to [zed-industries/extensions](https://github.com/zed-industries/extensions),
adding this repo as a submodule and pointing each entry at its subfolder:

```toml
[my-extension]
submodule = "extensions/<this-repo>"
path = "extensions/my-extension"
version = "0.1.0"
```

## License

[MIT](LICENSE)
