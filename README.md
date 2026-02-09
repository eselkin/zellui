# Zellui (Zell(ij) (t)UI)

A powerful terminal user interface for managing [Zellij] sessions, built with [Ratatui].

## Features

- **Session Management**: Easily attach to, kill, or delete Zellij sessions.
- **Intelligent Filtering**:
  - **Attach/Delete**: Shows only exited sessions.
  - **Kill**: Shows only active sessions.
- **New Session Creation**:
  - Browse and select from your layouts and configurations directly from the TUI.
  - Dual-list picker for layouts and configs.
  - TODO: allow args to be typed in
- **Optimistic Updates**: Immediate UI feedback when sessions are killed or deleted.
- **Responsive Navigation Bar**: Adapts to terminal size, switching to a compact view when the width is below 172 columns.

## Navigation & Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle through actions (Attach, Kill, Delete, New Session) |
| `A`, `K`, `D`, `N` | Direct shortcuts to actions |
| `Left` / `Right` | Switch focus between Layout and Config lists (New Session mode) |
| `Up` / `Down` | Select items in the current list |
| `Enter` | Execute the selected action |
| `G` | Jump to a specific index (Go to). Why? Because what if you have 100 sessions? |
| `R` | Refresh session and file lists |
| `Q` | Quit Zellui |

## Running

```bash
cargo run
```

## Building

```bash
cargo build -r
```

## License

Copyright (c) Eli Selkin <5606931+eselkin@users.noreply.github.com>

Licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>).

[Zellij]: https://zellij.dev
[Ratatui]: https://ratatui.rs
