mod host_info;
mod line;
mod tab;

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;

use host_info::{parse_hostname, parse_ipv4_address, HOSTNAME_COMMAND, IP_COMMANDS};
use tab::get_tab_to_focus;
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

use crate::line::{tab_line, tab_line_suffix};
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
    hostname: String,
    ip_address: String,
    host_info_requested: bool,
    clock: String,
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
    display_host_info: bool,
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
            display_host_info: Self::get_bool_from_configuration(
                configuration,
                "DisplayHostInfo",
                false,
            ),
        }
    }
}

fn request_host_info() {
    let mut hostname_context = BTreeMap::new();
    hostname_context.insert("type".to_string(), "hostname".to_string());
    run_command(HOSTNAME_COMMAND, hostname_context);

    for command in IP_COMMANDS {
        let mut ip_context = BTreeMap::new();
        ip_context.insert("type".to_string(), "ip_address".to_string());
        run_command(command, ip_context);
    }
}

fn request_clock() {
    // `run_command` executes the program directly on the host. Keep both
    // commands here because the plugin is one WASM artifact used on Unix and
    // Windows; Windows does not provide GNU `date`.
    let mut unix_clock_context = BTreeMap::new();
    unix_clock_context.insert("type".to_string(), "clock".to_string());
    run_command(&["date", "+%u %Y-%m-%d %H:%M"], unix_clock_context);

    let mut windows_clock_context = BTreeMap::new();
    windows_clock_context.insert("type".to_string(), "clock".to_string());
    run_command(
        &[
            "powershell.exe",
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "'{0} {1:yyyy-MM-dd HH:mm}' -f ((([int](Get-Date).DayOfWeek + 6) % 7) + 1), (Get-Date)",
        ],
        windows_clock_context,
    );
}

fn update_host_info(state: &mut State, stdout: &[u8], context: &BTreeMap<String, String>) -> bool {
    match context.get("type").map(String::as_str) {
        Some("hostname") => parse_hostname(stdout)
            .map(|hostname| state.hostname = hostname)
            .is_some(),
        Some("ip_address") => parse_ipv4_address(stdout)
            .map(|ip_address| state.ip_address = ip_address)
            .is_some(),
        _ => false,
    }
}

fn update_clock(state: &mut State, stdout: &[u8], context: &BTreeMap<String, String>) -> bool {
    if context.get("type").map(String::as_str) != Some("clock") {
        return false;
    }
    let Some(clock) = format_clock(stdout) else {
        return false;
    };
    if clock == state.clock {
        return false;
    }
    state.clock = clock;
    true
}

fn format_clock(stdout: &[u8]) -> Option<String> {
    let raw = String::from_utf8_lossy(stdout).trim().to_string();
    let (weekday, datetime) = raw.split_once(' ')?;
    let weekday = weekday.parse::<u8>().ok()?;
    if !(1..=7).contains(&weekday) || !is_valid_datetime(datetime) {
        return None;
    }
    let weekday_name = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ]
    .get((weekday - 1) as usize)?;
    Some(format!("{weekday_name} {datetime}"))
}

fn is_valid_datetime(datetime: &str) -> bool {
    datetime.len() == 16
        && datetime
            .as_bytes()
            .iter()
            .enumerate()
            .all(|(index, byte)| match index {
                4 | 7 => *byte == b'-',
                10 => *byte == b' ',
                13 => *byte == b':',
                _ => byte.is_ascii_digit(),
            })
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.configuration = configuration;
        let permissions = vec![
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ];
        request_permission(&permissions);
        subscribe(&[
            EventType::TabUpdate,
            EventType::ModeUpdate,
            EventType::Mouse,
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
            EventType::Timer,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::RunCommandResult(_exit_code, stdout, _stderr, context) => {
                should_render = update_host_info(self, &stdout, &context)
                    || update_clock(self, &stdout, &context);
            }
            Event::Timer(_) => {
                request_clock();
                set_timeout(60.0);
            }
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
                if UserConfiguration::get_bool_from_configuration(
                    &self.configuration,
                    "DisplayHostInfo",
                    false,
                ) && !self.host_info_requested
                {
                    request_host_info();
                    self.host_info_requested = true;
                }
                request_clock();
                set_timeout(60.0);
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

        // Reserve 2 chars for left/right padding.
        let usable_cols = cols.saturating_sub(2);
        let clock = get_clock(self.clock.clone(), self.user_configuration.clone());
        let host_line = if self.user_configuration.display_host_info {
            tab_line_suffix(
                self.hostname.clone(),
                self.ip_address.clone(),
                usable_cols,
                self.user_configuration.clone(),
            )
            .into_iter()
            .next()
        } else {
            None
        };
        let host_width = host_line.as_ref().map_or(0, |part| part.len);
        let host_gap = if host_width > 0 { 2 } else { 0 };
        let reserved_right_width = clock.len + host_gap + host_width;

        // Build left side (session name + tabs), leaving room for hints and host info.
        let left_line = tab_line(
            self.mode_info.session_name.clone().unwrap_or_default(),
            all_tabs,
            active_tab_index,
            usable_cols.saturating_sub(reserved_right_width),
            self.user_configuration.clone(),
            self.mode_info.mode,
            String::new(),
        );
        self.tab_line = left_line;

        let left_width: usize = self.tab_line.iter().map(|p| p.len).sum();
        let has_space_for_right = usable_cols.saturating_sub(left_width) >= reserved_right_width;

        let background = self.user_configuration.color_bg;
        // Apply background color to padding area.
        let bg_escape = match background {
            PaletteColor::Rgb((r, g, b)) => format!("\u{1b}[48;2;{};{};{}m", r, g, b),
            PaletteColor::EightBit(color) => format!("\u{1b}[48;5;{}m", color),
        };

        let left_output: String = self
            .tab_line
            .iter()
            .map(|p| format!("{}{}", p.part, bg_escape))
            .collect();
        let output = if has_space_for_right {
            let padding = usable_cols.saturating_sub(left_width + reserved_right_width);
            let host_part = host_line.map_or_else(String::new, |part| {
                format!("{}{}{}", " ".repeat(host_gap), part.part, bg_escape)
            });
            format!(
                " {}{}{}{}{}{} ",
                left_output,
                bg_escape,
                " ".repeat(padding),
                clock.part,
                bg_escape,
                host_part
            )
        } else {
            let padding = usable_cols.saturating_sub(left_width);
            format!(" {}{}{} ", left_output, bg_escape, " ".repeat(padding))
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

fn get_clock(clock: String, user_conf: UserConfiguration) -> LinePart {
    let bg_color = user_conf.color_bg;
    let fg_color = user_conf.color_fg;
    let len = clock.len();
    let styled = style!(fg_color, bg_color).bold().paint(clock);
    LinePart {
        part: styled.to_string(),
        len,
        tab_index: None,
    }
}
