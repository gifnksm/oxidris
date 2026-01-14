use ratatui::{layout::Rect, widgets::Block as BlockWidget};

pub use self::{
    block_display::*, board_display::*, piece_display::*, piece_stack_display::*,
    session_display::*, session_stats_display::*,
};

mod block_display;
mod board_display;
mod piece_display;
mod piece_stack_display;
mod session_display;
mod session_stats_display;

mod color {
    use ratatui::style::Color;

    // Common colors as associated constants
    pub const CYAN: Color = Color::Rgb(0, 255, 255);
    pub const YELLOW: Color = Color::Rgb(255, 255, 0);
    pub const GREEN: Color = Color::Rgb(0, 255, 0);
    pub const RED: Color = Color::Rgb(255, 0, 0);
    pub const BLUE: Color = Color::Rgb(0, 0, 255);
    pub const ORANGE: Color = Color::Rgb(255, 127, 0);
    pub const MAGENTA: Color = Color::Rgb(255, 0, 255);
    pub const GRAY: Color = Color::Rgb(127, 127, 127);
    pub const BLACK: Color = Color::Rgb(0, 0, 0);
    pub const WHITE: Color = Color::Rgb(255, 255, 255);
}

pub mod style {
    use ratatui::style::{Color, Style};

    use crate::ui::widgets::color;

    const fn fg_bg(fg: Color, bg: Color) -> Style {
        Style::new().fg(fg).bg(bg)
    }

    const fn bg_only(color: Color) -> Style {
        Style::new().fg(color).bg(color)
    }

    pub const DEFAULT: Style = fg_bg(color::WHITE, color::BLACK);
    pub const EMPTY: Style = bg_only(color::BLACK);
    pub const EMPTY_DOT: Style = fg_bg(color::GRAY, color::BLACK);
    pub const WALL: Style = bg_only(color::GRAY);
    pub const GHOST: Style = fg_bg(color::WHITE, color::BLACK);

    pub const I_BLOCK: Style = bg_only(color::CYAN);
    pub const O_BLOCK: Style = bg_only(color::YELLOW);
    pub const S_BLOCK: Style = bg_only(color::GREEN);
    pub const Z_BLOCK: Style = bg_only(color::RED);
    pub const J_BLOCK: Style = bg_only(color::BLUE);
    pub const L_BLOCK: Style = bg_only(color::ORANGE);
    pub const T_BLOCK: Style = bg_only(color::MAGENTA);
}

fn block_vertical_margin(block: Option<&BlockWidget>) -> u16 {
    let dummy_rect = Rect::new(0, 0, 100, 100);
    let inner_rect = block.map_or(dummy_rect, |block| block.inner(dummy_rect));
    dummy_rect.height - inner_rect.height
}

fn block_horizontal_margin(block: Option<&BlockWidget>) -> u16 {
    let dummy_rect = Rect::new(0, 0, 100, 100);
    let inner_rect = block.map_or(dummy_rect, |block| block.inner(dummy_rect));
    dummy_rect.width - inner_rect.width
}
