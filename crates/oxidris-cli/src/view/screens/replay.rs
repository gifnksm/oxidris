use std::{path::PathBuf, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEvent};
use oxidris_engine::{Block, BlockBoard};
use ratatui::{
    Frame,
    layout::{Constraint, HorizontalAlignment, Layout, Spacing},
    style::Color,
    symbols::merge::MergeStrategy,
    text::{Line, Text},
    widgets::{Block as BlockWidget, Padding, Paragraph},
};

use crate::{
    DEFAULT_FRAME_RATE,
    schema::record::{RecordedSession, TurnRecord},
    tui::{RenderMode, Screen, ScreenTransition, Tui},
    view::widgets::{BoardDisplay, KeyBinding, KeyBindingDisplay},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    TogglePlay,
    Prev(usize),
    Next(usize),
    First,
    Last,
    Quit,
}

impl Action {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Char(' ') => Some(Self::TogglePlay),
            KeyCode::Char('k') | KeyCode::Up => Some(Self::Prev(1)),
            KeyCode::Char('j') | KeyCode::Down => Some(Self::Next(1)),
            KeyCode::Char('h') | KeyCode::Left => Some(Self::Prev(10)),
            KeyCode::Char('l') | KeyCode::Right => Some(Self::Next(10)),
            KeyCode::Char('g') | KeyCode::Home => Some(Self::First),
            KeyCode::Char('G') | KeyCode::End => Some(Self::Last),
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings() -> &'static [KeyBinding<'static>] {
        &[
            (&["Space"], "Play/Pause"),
            (&["k", "↑"], "Prev"),
            (&["j", "↓"], "Next"),
            (&["h", "←"], "Prev 10"),
            (&["l", "→"], "Next 10"),
            (&["g", "Home"], "First"),
            (&["G", "End"], "Last"),
            (&["q", "Esc"], "Quit"),
        ]
    }
}

#[derive(Debug)]
pub struct ReplayScreen {
    path: PathBuf,
    session: RecordedSession,
    board_index: usize,
    play: bool,
}

impl ReplayScreen {
    pub fn new(path: PathBuf, session: RecordedSession) -> Self {
        Self {
            path,
            session,
            board_index: 0,
            play: false,
        }
    }
}

impl Screen for ReplayScreen {
    fn on_active(&mut self, tui: &mut Tui) {
        tui.set_render_mode(RenderMode::throttled_from_rate(DEFAULT_FRAME_RATE));
        self.update_tick_interval(tui);
    }

    fn on_inactive(&mut self, _tui: &mut Tui) {}

    fn on_close(&mut self, _tui: &mut Tui) {}

    fn handle_event(&mut self, tui: &mut Tui, event: &Event) -> ScreenTransition {
        if let Some(event) = event.as_key_event()
            && let Some(action) = Action::from_key_event(&event)
        {
            match action {
                Action::TogglePlay => {
                    self.play = !self.play;
                    self.update_tick_interval(tui);
                }
                Action::Prev(amount) => self.step_backward(amount),
                Action::Next(amount) => self.step_forward(amount),
                Action::First => self.jump_to_first(),
                Action::Last => self.jump_to_last(),
                Action::Quit => return ScreenTransition::Pop,
            }
        }
        ScreenTransition::Stay
    }

    fn update(&mut self, _tui: &mut Tui) {
        assert!(self.play);
        self.step_forward(1);
    }

    fn draw(&self, frame: &mut Frame) {
        let top_block = BlockWidget::bordered()
            .title(format!("Replay: {}", self.path.display()))
            .title_alignment(HorizontalAlignment::Center)
            .padding(Padding::symmetric(1, 0));
        let viewport = frame
            .area()
            .centered(Constraint::Max(80 * 2), Constraint::Max(60));

        if self.session.boards.is_empty() {
            let text_area = top_block
                .inner(viewport)
                .centered_vertically(Constraint::Length(1));
            let text = Text::from("NO BOARDS AVAILABLE")
                .centered()
                .style(Color::Red);
            frame.render_widget(top_block, viewport);
            frame.render_widget(text, text_area);
            return;
        }

        let stats = &self.session.final_stats;
        let record = &self.session.boards[self.board_index];

        let [top_area, mid_area, bottom_area] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .spacing(Spacing::Overlap(1))
        .areas(viewport);

        let stats = Paragraph::new(vec![
            Line::from(format!(
                "Index: {:8}/{:8}",
                self.board_index,
                self.session.boards.len()
            )),
            Line::from(format!("Turn:  {:8}/{:8}", record.turn, stats.turn())),
        ])
        .block(top_block.merge_borders(MergeStrategy::Exact));

        let block_board = create_block_board(record);
        let board_display = BoardDisplay::new(&block_board).block(
            BlockWidget::bordered()
                .padding(Padding::symmetric(1, 0))
                .merge_borders(MergeStrategy::Exact),
        );

        let help = KeyBindingDisplay::new(Action::bindings())
            .block(BlockWidget::bordered().merge_borders(MergeStrategy::Exact));

        frame.render_widget(stats, top_area);
        frame.render_widget(board_display, mid_area);
        frame.render_widget(help, bottom_area);
    }
}

impl ReplayScreen {
    fn update_tick_interval(&mut self, tui: &mut Tui) {
        let interval = self.play.then(|| Duration::from_millis(100));
        tui.set_tick_interval(interval);
    }

    fn step_forward(&mut self, amount: usize) {
        let len = self.session.boards.len();
        if len == 0 {
            return;
        }
        self.board_index = usize::min(self.board_index + amount, len - 1);
    }

    fn step_backward(&mut self, amount: usize) {
        self.board_index = self.board_index.saturating_sub(amount);
    }

    fn jump_to_first(&mut self) {
        self.board_index = 0;
    }

    fn jump_to_last(&mut self) {
        self.board_index = self.session.boards.len().saturating_sub(1);
    }
}

fn create_block_board(record: &TurnRecord) -> BlockBoard {
    let mut board = BlockBoard::INITIAL;
    for (x, y) in record.before_placement.occupied_cell_positions() {
        board.fill_block_at(x, y, Block::Wall);
    }
    board.fill_piece(record.placement);
    board
}
