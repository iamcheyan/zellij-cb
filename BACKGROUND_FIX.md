# zellij-cb 状态栏背景色问题

## 问题描述

在修改 zellij-cb 插件以模仿 tmux 状态栏样式时，遇到一个顽固的背景色问题：

**状态栏中间的 padding 区域和右侧边缘无法正确显示绿色背景，会出现黑色区域。**

## 期望效果

整个状态栏从左到右应该全部是绿色背景（ANSI bright green, `\e[48;5;10m` 或 RGB `0,255,0`），包括：
- 左侧内容（session 名 + tabs）
- 中间 padding 空格
- 右侧内容（hostname + 时间 + 日期）
- 最右侧的空白填充区域

## 当前现象

1. 左侧内容区域：绿色背景 ✅
2. 中间 padding 区域：黑色背景 ❌
3. 右侧内容区域：绿色背景 ✅
4. 最右侧空白区域：黑色背景 ❌

## 已尝试的方案

### 方案 1：在 output 开头设置背景色
```rust
print!("\u{1b}[48;2;{};{};{}m{}\u{1b}[0K", r, g, b, output);
```
**结果**：左侧有绿色，中间和右侧仍然黑色。

### 方案 2：在 padding 前添加 bg_escape
```rust
let output = format!(" {}{}{}{} ", left_output, bg_escape, padding_str, right_output);
```
**结果**：中间 padding 有绿色，但最右侧仍然是黑色。

### 方案 3：用 \e[0K 清行
```rust
print!("{}\u{1b}[48;2;{};{};{}m\u{1b}[0K", output, r, g, b);
```
**结果**：`\e[0K` 使用终端默认背景色（黑色）清除行尾，导致最右侧变黑。

### 方案 4：用绿色空格填充代替 \e[0K
```rust
let fill_str = " ".repeat(fill_width);
let output = format!(" {}{}{}{}{} ", left_output, bg_escape, padding_str, right_output, fill_str);
print!("\u{1b}[48;2;{};{};{}m{}", r, g, b, output);
```
**结果**：填充区域有绿色，但 `style!` 宏生成的样式文本在打印后会重置背景色。

## 根本原因分析

`zellij_tile_utils::style!` 宏生成的 ANSI 样式文本包含颜色设置和重置序列。当打印 `style!(fg, bg).paint(text)` 生成的字符串时：

1. 设置前景色和背景色
2. 打印文本
3. **重置所有颜色**（包括背景色）

这导致每个 `LinePart` 打印后，背景色被重置为终端默认色（黑色），后续的 padding 和 fill 无法继承绿色背景。

## 可能的解决方案

1. **在每个 LinePart 后重新设置背景色**：遍历每个 part，在其后附加 `bg_escape`
2. **不使用 `style!` 宏**：手动构建 ANSI 转义序列，避免颜色重置
3. **使用 `\e[48;2;R;G;Bm` 设置背景后不重置**：确保整个输出都在同一个背景色下

## 相关代码位置

- `src/main.rs` - `render()` 函数（~行 301-370）
- `src/line.rs` - `tab_line()` 和 `tab_line_suffix()` 函数
- `src/tab.rs` - `render_tab()` 函数中使用 `style!` 宏

## 参考

- tmux 状态栏样式：绿底黑字，左右布局
- ANSI 转义序列：`\e[48;5;Nm` (8-bit) 或 `\e[48;2;R;G;Bm` (RGB)
- zellij-tile-utils crate 中的 `style!` 宏
