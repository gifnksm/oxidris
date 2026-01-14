use oxidris_engine::{GameSession, SessionState};
use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::{Buffer, Rect},
    style::Style,
    text::{Line, Text},
    widgets::{Block, Clear, Padding, Widget},
};

use crate::ui::widgets::{
    BoardDisplay, PieceDisplay, PieceStackDisplay, SessionStatsDisplay, color, style,
};

#[derive(Debug)]
pub struct SessionDisplay<'a> {
    session: &'a GameSession,
    show_ghost: bool,
    turbo: bool,
    horizontal_padding: u16,
    vertical_padding: u16,
    next_pieces: usize,
}

impl<'a> SessionDisplay<'a> {
    pub fn new(session: &'a GameSession, show_ghost: bool) -> Self {
        Self {
            session,
            show_ghost,
            turbo: false,
            horizontal_padding: 1,
            vertical_padding: 0,
            next_pieces: 7,
        }
    }

    pub fn turbo(self, turbo: bool) -> Self {
        Self { turbo, ..self }
    }
}

impl Widget for SessionDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &SessionDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = style::DEFAULT;
        let block_padding = Padding::symmetric(self.horizontal_padding, self.vertical_padding);
        let border_style = match self.session.session_state() {
            SessionState::Playing if self.turbo => color::MAGENTA,
            SessionState::Playing => color::WHITE,
            SessionState::Paused => color::YELLOW,
            SessionState::GameOver => color::RED,
        };

        let game_board = {
            let widget = BoardDisplay::new(self.session.block_board())
                .falling_piece(self.session.falling_piece())
                .block(Block::bordered().border_style(border_style).style(style));
            if self.show_ghost {
                widget.ghost(self.session.simulate_drop_position())
            } else {
                widget
            }
        };
        let hold_panel = {
            let panel = PieceDisplay::new().block(
                Block::bordered()
                    .title(Line::from("HOLD").centered())
                    .padding(block_padding)
                    .border_style(border_style)
                    .style(style::DEFAULT),
            );
            if let Some(piece) = self.session.held_piece() {
                panel.piece(piece)
            } else {
                panel
            }
        };
        let piece_stack = PieceStackDisplay::new(self.session.next_pieces().take(self.next_pieces))
            .block(
                Block::bordered()
                    .title(Line::from("NEXT").centered())
                    .padding(block_padding)
                    .border_style(border_style)
                    .style(style::DEFAULT),
            );
        let session_stats = SessionStatsDisplay::new(self.session).block(
            Block::bordered()
                .title(Line::from("STATS").centered())
                .padding(block_padding)
                .border_style(border_style)
                .style(style::DEFAULT),
        );

        let [left_column, center_column, right_column] = Layout::horizontal([
            Constraint::Length(u16::max(hold_panel.width(), session_stats.width())),
            Constraint::Length(game_board.width()),
            Constraint::Length(piece_stack.width()),
        ])
        .flex(Flex::Center)
        .spacing(1)
        .areas(area);

        let [hold_area, stats_area] = Layout::vertical([
            Constraint::Length(hold_panel.height()),
            Constraint::Length(session_stats.height()),
        ])
        .spacing(1)
        .areas(left_column);
        let hold_area = hold_area.layout::<1>(
            &Layout::horizontal([Constraint::Length(hold_panel.width())]).flex(Flex::End),
        )[0];
        let stats_area = stats_area.layout::<1>(
            &Layout::horizontal([Constraint::Length(session_stats.width())]).flex(Flex::End),
        )[0];

        let [board_area] =
            Layout::vertical([Constraint::Length(game_board.height())]).areas(center_column);

        let [piece_stack_area] =
            Layout::vertical([Constraint::Length(piece_stack.height())]).areas(right_column);

        let game_board_width = game_board.width();
        hold_panel.render(hold_area, buf);
        session_stats.render(stats_area, buf);
        game_board.render(board_area, buf);
        piece_stack.render(piece_stack_area, buf);

        let popup = match self.session.session_state() {
            SessionState::Playing => None,
            SessionState::Paused => {
                Some(("PAUSED", Style::new().fg(color::BLACK).bg(color::YELLOW)))
            }
            SessionState::GameOver => {
                Some(("GAME OVER!!", Style::new().fg(color::WHITE).bg(color::RED)))
            }
        };

        if let Some((text, style)) = popup {
            let block = Block::new().style(style);
            let text = Text::styled(text, style).centered();
            let area =
                board_area.centered(Constraint::Length(game_board_width), Constraint::Length(3));
            let inner = block.inner(area);
            Clear.render(area, buf);
            block.render(area, buf);
            text.render(inner.centered_vertically(Constraint::Length(1)), buf);
        }
    }
}
