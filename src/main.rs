mod line;
mod tab;

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;

use tab::get_tab_to_focus;
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

use crate::line::tab_line;
use crate::tab::tab_style;

#[derive(Debug, Default)]
pub struct LinePart {
    part: String,
    len: usize,
    tab_index: Option<usize>,
}

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    active_tab_idx: usize,
    configuration: BTreeMap<String, String>,
    user_configuration: UserConfiguration,
    mode_info: ModeInfo,
    tab_line: Vec<LinePart>,
}

register_plugin!(State);

#[derive(Default, Clone, Debug)]
pub struct UserConfiguration {
    color_fg: PaletteColor,
    color_bg: PaletteColor,
    color_session_directory: PaletteColor,
    color_session_name: PaletteColor,
    color_tab: PaletteColor,
    color_active_tab: PaletteColor,
    color_normal_mode: PaletteColor,
    color_other_modes: PaletteColor,
    color_others: PaletteColor,
    display_session_directory: bool,
    default_tab_name: String,
    mode_display: HashMap<InputMode, String>,
}

impl UserConfiguration {
    fn str_to_palette_color(color_str: &str) -> Option<PaletteColor> {
        let color_parts = color_str
            .split(",")
            .filter_map(|part| part.parse::<u8>().ok())
            .collect::<Vec<_>>();
        Some(match color_parts.len() {
            1 => PaletteColor::EightBit(color_parts[0]),
            3 => PaletteColor::Rgb((color_parts[0], color_parts[1], color_parts[2])),
            _ => {
                eprintln!("{color_str} is not a valid color");
                return None;
            }
        })
    }
    fn get_color_from_configuration(
        configuration: &BTreeMap<String, String>,
        color_query: &str,
        fallback_color: PaletteColor,
    ) -> PaletteColor {
        if let Some(color_string) = configuration.get(color_query) {
            if let Some(color) = Self::str_to_palette_color(color_string) {
                return color;
            }
        }
        fallback_color
    }
    fn get_string_from_configuration(
        configuration: &BTreeMap<String, String>,
        query: &str,
        fallback: &str,
    ) -> String {
        match configuration.get(query) {
            Some(value) => value,
            None => fallback,
        }
        .to_string()
    }
    fn get_bool_from_configuration(
        configuration: &BTreeMap<String, String>,
        query: &str,
        fallback: bool,
    ) -> bool {
        match configuration.get(query) {
            Some(value) => value.parse().unwrap_or(fallback),
            None => fallback,
        }
    }
    pub fn populate_from_configuration(
        configuration: &BTreeMap<String, String>,
        _colors: &Styling,
    ) -> Self {
        let mode_display: HashMap<InputMode, String> = [
            InputMode::Normal,
            InputMode::Locked,
            InputMode::Resize,
            InputMode::Pane,
            InputMode::Tab,
            InputMode::Scroll,
            InputMode::EnterSearch,
            InputMode::Search,
            InputMode::RenameTab,
            InputMode::RenamePane,
            InputMode::Session,
            InputMode::Move,
            InputMode::Prompt,
            InputMode::Tmux,
        ]
        .iter()
        .cloned()
        .map(|mode| {
            let mode_string = format!("{:?}", mode);
            let fallback = if mode == InputMode::Locked {
                String::new()
            } else {
                mode_string.chars().next().unwrap().to_uppercase().collect()
            };
            (
                mode,
                Self::get_string_from_configuration(
                    configuration,
                    format!("{mode_string}ModeLabel").as_str(),
                    &fallback,
                ),
            )
        })
        .collect();

        // Tmux-like colors: black text on green background
        let tmux_green = PaletteColor::EightBit(10); // Bright green (ANSI 10)
        let tmux_black = PaletteColor::EightBit(0); // Black (ANSI 0)

        Self {
            mode_display,
            color_fg: Self::get_color_from_configuration(configuration, "FgColor", tmux_black),
            color_bg: Self::get_color_from_configuration(configuration, "BgColor", tmux_green),
            color_session_directory: Self::get_color_from_configuration(
                configuration,
                "SessionDirectoryColor",
                tmux_black,
            ),
            color_session_name: Self::get_color_from_configuration(
                configuration,
                "SessionNameColor",
                tmux_black,
            ),
            color_tab: Self::get_color_from_configuration(configuration, "TabColor", tmux_black),
            color_active_tab: Self::get_color_from_configuration(
                configuration,
                "ActiveTabColor",
                tmux_black,
            ),
            color_normal_mode: Self::get_color_from_configuration(
                configuration,
                "NormalModeColor",
                tmux_black,
            ),
            color_other_modes: Self::get_color_from_configuration(
                configuration,
                "OtherModesColor",
                tmux_black,
            ),
            color_others: Self::get_color_from_configuration(
                configuration,
                "OthersColor",
                tmux_black,
            ),
            default_tab_name: Self::get_string_from_configuration(
                configuration,
                "DefaultTabName",
                "tab",
            ),
            display_session_directory: Self::get_bool_from_configuration(
                configuration,
                "DisplaySessionDirectory",
                true,
            ),
        }
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::TabUpdate,
            EventType::ModeUpdate,
            EventType::Mouse,
            EventType::PermissionRequestResult,
        ]);
        self.configuration = _configuration;
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::ModeUpdate(mode_info) => {
                self.user_configuration = UserConfiguration::populate_from_configuration(
                    &self.configuration,
                    &mode_info.style.colors,
                );
                self.mode_info = mode_info;
                should_render = true;
            }
            Event::TabUpdate(tabs) => {
                self.active_tab_idx = tabs.iter().position(|t| t.active).unwrap_or(0) + 1;
                self.tabs = tabs;
                should_render = true;
            }
            Event::Mouse(me) => match me {
                Mouse::LeftClick(_, col) => {
                    let tab_to_focus = get_tab_to_focus(&self.tab_line, self.active_tab_idx, col);
                    if let Some(idx) = tab_to_focus {
                        switch_tab_to(idx.try_into().unwrap());
                    }
                }
                Mouse::ScrollUp(_) => {
                    switch_tab_to(min(self.active_tab_idx + 1, self.tabs.len()) as u32);
                }
                Mouse::ScrollDown(_) => {
                    switch_tab_to(max(self.active_tab_idx.saturating_sub(1), 1) as u32);
                }
                _ => {}
            },
            Event::PermissionRequestResult(_) => {
                set_selectable(false);
            }
            _ => {
                eprintln!("Got unrecognized event: {:?}", event);
            }
        };
        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        if self.tabs.is_empty() {
            return;
        }
        let mut all_tabs: Vec<LinePart> = vec![];
        let mut active_tab_index = 0;
        let mut is_alternate_tab = false; // NOTE: In case I need it in the future
        for t in &mut self.tabs {
            let mut tabname = t.name.clone();
            if t.active && self.mode_info.mode == InputMode::RenameTab {
                if tabname.is_empty() {
                    tabname = String::from("Enter name...");
                }
                active_tab_index = t.position;
            } else if t.active {
                active_tab_index = t.position;
            }
            let tab = tab_style(tabname, t, self.user_configuration.clone());
            is_alternate_tab = !is_alternate_tab;
            all_tabs.push(tab);
        }

        // Reserve 2 chars for left/right padding
        let usable_cols = cols.saturating_sub(2);

        // Build right side (hostname, time, date) - disabled as requested
        let right_width: usize = 0;

        // Build left side (session name + tabs)
        let left_line = tab_line(
            self.mode_info.session_name.clone().unwrap_or_default(),
            all_tabs,
            active_tab_index,
            usable_cols.saturating_sub(right_width),
            self.user_configuration.clone(),
            self.mode_info.mode,
            String::new(),
        );
        self.tab_line = left_line;

        let left_width: usize = self.tab_line.iter().map(|p| p.len).sum();

        let mode_hint = get_mode_hint(self.mode_info.mode, self.user_configuration.clone());
        let mode_hint_len = mode_hint.len;

        // If there's enough space, align the mode hint to the very right
        let has_space_for_hint = usable_cols.saturating_sub(left_width) >= mode_hint_len + 2; // at least 2 spaces padding

        let background = self.user_configuration.color_bg;
        // Apply background color to padding area
        let bg_escape = match background {
            PaletteColor::Rgb((r, g, b)) => format!("\u{1b}[48;2;{};{};{}m", r, g, b),
            PaletteColor::EightBit(color) => format!("\u{1b}[48;5;{}m", color),
        };

        let (padding_str, hint_part) = if has_space_for_hint {
            let padding = usable_cols.saturating_sub(left_width + mode_hint_len);
            (" ".repeat(padding), mode_hint.part)
        } else {
            let padding = usable_cols.saturating_sub(left_width);
            (" ".repeat(padding), "".to_string())
        };

        // Combine left
        // Re-apply bg after each styled part since style! resets it
        let left_output: String = self
            .tab_line
            .iter()
            .map(|p| format!("{}{}", p.part, bg_escape))
            .collect();

        let output = if has_space_for_hint {
            format!(
                " {}{}{}{}{} ",
                left_output, bg_escape, padding_str, hint_part, bg_escape
            )
        } else {
            format!(" {}{}{} ", left_output, bg_escape, padding_str)
        };

        match background {
            PaletteColor::Rgb((r, g, b)) => {
                print!("\u{1b}[48;2;{};{};{}m{}", r, g, b, output);
            }
            PaletteColor::EightBit(color) => {
                print!("\u{1b}[48;5;{}m{}", color, output);
            }
        }
    }
}

fn get_mode_hint(mode: InputMode, user_conf: UserConfiguration) -> LinePart {
    let bg_color = user_conf.color_bg;
    let fg_color = user_conf.color_fg;

    let text = match mode {
        InputMode::Normal => "g:LOCK p:PANE t:TAB n:RESIZE h:MOVE s:SCROLL o:SESSION",
        InputMode::Locked => "g:UNLOCK",
        InputMode::Pane => "[PANE] n:New d:Down r:Right x:Close f:Full p:Next h/j/k/l:Move",
        InputMode::Tab => "[TAB] n:New x:Close r:Rename h/l:Move s:Sync",
        InputMode::Resize => "[RESIZE] h/j/k/l or +/-: Resize",
        InputMode::Move => "[MOVE] h/j/k/l: Move Pane",
        InputMode::Scroll => "[SCROLL] u/d: Half Pg U/D Up/Down /: Search",
        InputMode::Search => "[SEARCH] Enter: Search n: Next p: Prev",
        InputMode::EnterSearch => "[SEARCH] Type term, then press Enter",
        InputMode::RenameTab => "[RENAME TAB] Type name, then press Enter",
        InputMode::RenamePane => "[RENAME PANE] Type name, then press Enter",
        InputMode::Session => "[SESSION] d: Detach w: Managers",
        InputMode::Tmux => "[TMUX] d: Detach ?: Help",
        _ => "Press Esc to exit mode",
    };

    let len = text.len();
    let styled = style!(fg_color, bg_color).bold().paint(text);
    LinePart {
        part: styled.to_string(),
        len,
        tab_index: None,
    }
}
