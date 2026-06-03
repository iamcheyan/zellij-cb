# zellij-cb

这是一个基于 [ndavd/zellij-cb](https://github.com/ndavd/zellij-cb) 的 fork。项目在保留原始实现与 MIT 许可的基础上，针对状态栏的视觉效果与交互细节做了若干调整，旨在为 Zellij 提供更接近 tmux 的使用体验。

![screenshot](image.png)

## 功能特点

- 以 tmux 风格展示标签：例如 `[1] name*`
- 使用绿色背景与深色文字，营造更接近 tmux 的视觉效果
- 在状态栏底部展示当前模式下可用的快捷键提示
- 在左侧显示会话名称，并优化布局与对齐表现

## 示例效果

```text
 main  [1] zsh*  [2] vim
 g:LOCK p:PANE t:TAB n:RESIZE h:MOVE s:SCROLL o:SESSION
```

## 安装说明

### 1. 构建插件

请确保已安装 Rust 工具链，并配置 `wasm32-wasip1` 目标。

```bash
make
```

生成结果位于：

```text
target/wasm32-wasip1/release/zellij-cb.wasm
```

### 2. 放置到 Zellij 插件目录

```bash
cp target/wasm32-wasip1/release/zellij-cb.wasm ~/.config/zellij/plugins/
```

## 配置方式

在 `~/.config/zellij/config.kdl` 中注册插件：

```kdl
plugins {
    zellij-cb location="file:~/.config/zellij/plugins/zellij-cb.wasm"
}
```

在布局中使用：

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

## 配置项

| 选项 | 说明 | 默认值 |
|---|---|---:|
| `DefaultTabName` | 未命名标签的默认名称 | `tab` |
| `DisplaySessionDirectory` | 是否在状态栏中显示会话目录 | `false` |
| `FgColor` | 前景色（8-bit 或 RGB） | `0` |
| `BgColor` | 背景色（8-bit 或 RGB） | `10` |

## 模式提示栏

底部提示栏会根据当前模式显示不同的快捷键提示：

| 模式 | 提示内容 |
|---|---|
| Normal | `g:LOCK p:PANE t:TAB n:RESIZE h:MOVE s:SCROLL o:SESSION` |
| Locked | `g:UNLOCK` |
| Pane | `[PANE] n:New d:Down r:Right x:Close f:Full p:Next` |
| Tab | `[TAB] n:New x:Close r:Rename h/l:Move s:Sync` |
| Resize | `[RESIZE] h/j/k/l or +/-: Resize` |
| Move | `[MOVE] h/j/k/l: Move Pane` |
| Scroll | `[SCROLL] u/d: Half Pg U/D Up/Down /: Search` |

## 说明

这个版本的目标并不是重写整个插件，而是在现有实现基础上做一组更贴近 tmux 风格的视觉与交互调整。对于希望在 Zellij 中获得更统一、更熟悉的状态栏体验的用户而言，这个 fork 是一个实用的选择。

## 致谢与许可

本项目基于原作者 [Nuno David](https://github.com/ndavd) 的实现开发，并保留原始 MIT 许可证。

