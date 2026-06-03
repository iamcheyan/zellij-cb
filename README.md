# zellij-cb

A [Zellij](https://zellij.xyz/) status bar plugin with a tmux-inspired look.

Fork of [ndavd/zellij-cb](https://github.com/ndavd/zellij-cb).

## Features

- Tmux-style tab display: `[1] name*` (active tab marked with `*`)
- Green background with black text (ANSI bright green)
- Mode hint bar at the bottom showing available keybindings
- Session name displayed on the left

## Screenshot

```
 main  [1] zsh*  [2] vim
 g:LOCK p:PANE t:TAB n:RESIZE h:MOVE s:SCROLL o:SESSION
```

## Installation

### Build from source

```bash
make
```

The compiled plugin will be at `target/wasm32-wasip1/release/zellij-cb.wasm`.

### Copy to Zellij plugins directory

```bash
cp target/wasm32-wasip1/release/zellij-cb.wasm ~/.config/zellij/plugins/
```

## Configuration

In your Zellij config (`~/.config/zellij/config.kdl`):

```kdl
plugins {
    zellij-cb location="file:~/.config/zellij/plugins/zellij-cb.wasm"
}
```

Use in a layout:

```kdl
layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij-cb" {
                DisplaySessionDirectory "false"
                DefaultTabName "tab"
            }
        }
    }
    tab name="main"
}
```

### Configuration options

| Option                  | Description                          | Default |
|-------------------------|--------------------------------------|---------|
| `DefaultTabName`        | Name for unnamed tabs                | `tab`   |
| `DisplaySessionDirectory` | Show session directory in bar      | `false` |
| `FgColor`               | Foreground color (8-bit or RGB)      | `0`     |
| `BgColor`               | Background color (8-bit or RGB)      | `10`    |

## Mode hints

The bottom bar shows keybindings for the current mode:

| Mode     | Hint                                                        |
|----------|-------------------------------------------------------------|
| Normal   | `g:LOCK p:PANE t:TAB n:RESIZE h:MOVE s:SCROLL o:SESSION`  |
| Locked   | `g:UNLOCK`                                                  |
| Pane     | `[PANE] n:New d:Down r:Right x:Close f:Full p:Next`        |
| Tab      | `[TAB] n:New x:Close r:Rename h/l:Move s:Sync`             |
| Resize   | `[RESIZE] h/j/k/l or +/-: Resize`                          |
| Move     | `[MOVE] h/j/k/l: Move Pane`                                |
| Scroll   | `[SCROLL] u/d: Half Pg U/D Up/Down /: Search`              |

## Credits

Original plugin by [Nuno David](https://github.com/ndavd).

## License

MIT
