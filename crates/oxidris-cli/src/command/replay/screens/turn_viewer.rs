use std::path::PathBuf;

use crossterm::event::{Event, KeyCode};
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
    schema::record::{RecordedSession, TurnRecord},
    view::widgets::BoardDisplay,
};

#[derive(Debug)]
pub struct TurnViewerScreen {
    path: PathBuf,
    session: RecordedSession,
    board_index: usize,
    should_exit: bool,
}

impl TurnViewerScreen {
    pub fn new(path: PathBuf, session: RecordedSession) -> Self {
        Self {
            path,
            session,
            board_index: 0,
            should_exit: false,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
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
                "Board: {:8}/{:8}",
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

        let help = Paragraph::new(vec![
            Line::from("j/k or ↓/↑ (1 turn) | h/l or ←/→ (10 turns) | g/Home (First) | G/End (Last) | q/Esc (Quit)").centered(),
        ]).style(Color::DarkGray)
        .block(BlockWidget::bordered().merge_borders(MergeStrategy::Exact));

        frame.render_widget(stats, top_area);
        frame.render_widget(board_display, mid_area);
        frame.render_widget(help, bottom_area);
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Some(event) = event.as_key_event() {
            match event.code {
                KeyCode::Char('j') | KeyCode::Down => self.step_forward(1),
                KeyCode::Char('k') | KeyCode::Up => self.step_backward(1),
                KeyCode::Char('h') | KeyCode::Left => self.step_backward(10),
                KeyCode::Char('l') | KeyCode::Right => self.step_forward(10),
                KeyCode::Char('g') | KeyCode::Home => self.jump_to_first(),
                KeyCode::Char('G') | KeyCode::End => self.jump_to_last(),
                KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                _ => {}
            }
        }
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
