# ferrisdex

Small terminal Pokedex app in Rust.

## CLI Options

| Flag                             | Meaning                                   | Notes                                                            |
|----------------------------------|-------------------------------------------|------------------------------------------------------------------|
| `shiny`                          | Request shiny sprite variant              | Positional toggle after pokemon name/id                          |
| `-c`, `--cache`                  | Enable HTTP cache                         | Overrides default `cache = false`                                |
| `-s`, `--sprite-source <source>` | Preferred sprite source                   | `sprites`, `home`, `official_artwork`, `showdown`, `dream_world` |
| `--female`                       | Prefer female sprite variant              | Falls back when female variant is unavailable                    |
| `--male`                         | Prefer default/male variant               | This is already the default                                      |
| `-g`, `--generation <gen>`       | Use generation sprite tree                | Accepts `i..ix` or `generation-i..generation-ix`                 |
| `--version-group <group>`        | Pick version group for generation sprites | Example: `platinum`, `black-white`, `x-y`                        |
| `--animated`                     | Prefer animated generation sprite path    | Applied only when that generation/version supports animation     |

## Cache Configuration

Caching is **off by default**.

You can enable it with either:

- CLI flag: `-c` or `--cache`
- Config file: 
  - Linux: `$HOME/.config/ferrisdex/config.toml`
  - Windows: `%APPDATA%\ferrisdex\config.toml`
  - macOS: `$HOME/Library/Application Support/ferrisdex/config.toml`

Example config:

```toml
cache = true
sprite_source = "official_artwork"
sprite_gender = "female"
sprite_generation = "generation-v"
sprite_version_group = "black-white"
sprite_animated = true
```

Or equivalently:

```toml
[cache]
enabled = true
```

Sprite options:

- `sprite_source`: `sprites | home | official_artwork | showdown | dream_world`
- CLI gender flags: `--female | --male` (`--male` is default)
- `sprite_gender`: `default | female`
- `sprite_generation`: `generation-i` .. `generation-ix`
- `sprite_version_group`: any version-group slug (for example `platinum`, `black-white`, `x-y`)
- `sprite_animated`: `true | false` (useful for generation sprites that provide animated variants)

CLI always overrides config. For example, if config sets `sprite_source = "home"`,
running with `--sprite-source showdown` will use showdown for that run.

Important behavior notes:

- `shiny` plus `--female` attempts female-shiny first, then falls back sensibly.
- When `-g` / `--generation` is set, FerrisDex builds the sprite URL from the versions tree.
- If `--version-group` is omitted, FerrisDex picks a default group per generation.
- Non-generation lookups try preferred source first, then fallback order: `official_artwork -> sprites -> dream_world -> home -> showdown`.

When cache is enabled, FerrisDex uses a shared HTTP cache for both:

- PokeAPI JSON responses (via `rustemon`)
- Sprite image downloads

Cache files are stored under:
- Linux: `$HOME/.cache/ferrisdex/http-cache/`.
- Windows: `%LOCALAPPDATA%\ferrisdex\http-cache\`.
- macOS: `$HOME/Library/Caches/ferrisdex/http-cache\`.



