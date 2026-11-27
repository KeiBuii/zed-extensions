# HTMX Language Server for Zed

Adds [htmx-lsp](https://github.com/ThePrimeagen/htmx-lsp) to Zed: completion and hover docs for `hx-*` attributes.

The server attaches alongside the existing language servers for:

HTML, PHP, Astro, ERB, HEEx, Templ, Go HTML Template, Jinja2, HTML-Jinja, Twig, Blade, Latte

Languages other than HTML come from their own extensions (e.g. `gotmpl`, `templ`, `twig`); install those too.

## Known limitation: press `Ctrl+Space` after `hx-`

Attribute suggestions do not pop up on their own. Type `hx-`, then press `Ctrl+Space`.

Why: htmx-lsp only answers once the attribute name starts with `hx-`, and waits for `-` as a trigger character.
Zed's HTML language treats `-` as part of the word being typed (`completion_query_characters = ["-"]`),
so it asks the server once at `h` (too early, empty result) and only filters locally after that.
Typing a space does trigger a request, but the accepted item is then inserted after the space and leaves a stray `hx-`.

On macOS, `Ctrl+Space` is usually taken by the system "Select the previous input source" shortcut. Either turn that off
(System Settings → Keyboard → Keyboard Shortcuts → Input Sources) or bind another key in `~/.config/zed/keymap.json`:

```json
[{ "context": "Editor", "bindings": { "ctrl-i": "editor::ShowCompletions" } }]
```

## Known limitation: no `=""` after the attribute (htmx-lsp 0.2.0)

The 0.2.0 release sends only the label, so accepting `hx-get` inserts `hx-get`. Upstream `master` (unreleased) inserts
`hx-get="…"` with the cursor between the quotes. To use it now, install it on your `PATH` and restart Zed:

```sh
cargo install --git https://github.com/ThePrimeagen/htmx-lsp --rev a05bf01d3371 htmx-lsp
```

## Snippets

Type `htmx_` in any of the languages above:

| Prefix | Inserts |
|---|---|
| `htmx_script` | `<script>` tag loading htmx 2 from jsDelivr |
| `htmx_get` / `htmx_put` / `htmx_patch` | element issuing that request into a target |
| `htmx_form` | form with `hx-post` |
| `htmx_delete` | delete button with `hx-confirm` |
| `htmx_search` | debounced active-search input |
| `htmx_infinite` | infinite scroll (`hx-trigger="revealed"`) |
| `htmx_lazy` | lazy load (`hx-trigger="load"`) |
| `htmx_poll` | polling (`hx-trigger="every 2s"`) |
| `htmx_edit` | click-to-edit button |
| `htmx_boost` | `hx-boost` container |
| `htmx_oob` | out-of-band swap fragment |
| `htmx_indicator` | loading indicator |
| `htmx_vals` / `htmx_headers` / `htmx_on` | attributes for extra values, headers, event handler |

`snippets/html.json` is the source; the other files are copies (Zed matches snippet files by language name). After editing it, re-copy:

```sh
cd snippets && for f in *.json; do [ "$f" = html.json ] || cp html.json "$f"; done
```

`cargo test` fails if a copy drifts or a language is missing a snippet file.

## Server binary

1. If `htmx-lsp` is on your `PATH`, that binary is used.
2. Otherwise the latest release binary is downloaded from GitHub (macOS arm64/x64, Linux x64, Windows arm64/x64).
3. On other platforms (e.g. Linux arm64), install it yourself: `cargo install htmx-lsp`.

## Turning it off for a language

```json
{
  "languages": {
    "PHP": { "language_servers": ["...", "!htmx-lsp"] }
  }
}
```
