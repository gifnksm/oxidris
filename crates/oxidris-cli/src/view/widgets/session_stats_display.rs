use std::iter;

use oxidris_engine::GameSession;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Block as BlockWidget, BlockExt as _, Widget},
};

use crate::view::widgets::style;

pub struct SessionStatsDisplay<'a> {
    session: &'a GameSession,
    block: Option<BlockWidget<'a>>,
}

impl<'a> SessionStatsDisplay<'a> {
    pub fn new(session: &'a GameSession) -> Self {
        Self {
            session,
            block: None,
        }
    }

    pub fn block(self, block: BlockWidget<'a>) -> Self {
        Self {
            block: Some(block),
            ..self
        }
    }

    pub fn width(&self) -> u16 {
        20 + super::block_horizontal_margin(self.block.as_ref())
    }

    pub fn height(&self) -> u16 {
        u16::try_from(ROWS.len()).unwrap() + super::block_vertical_margin(self.block.as_ref())
    }
}

#[derive(Clone, Copy)]
enum Row {
    Empty,
    FullLabel(&'static str),
    FullValue(&'static dyn Fn(&GameSession) -> String),
    LabelValue(&'static str, &'static dyn Fn(&GameSession) -> String),
}

const ROWS: &[Row] = &[
    Row::FullLabel("SCORE:"),
    Row::FullValue(&|session| session.stats().score().to_string()),
    Row::FullLabel("TIME:"),
    Row::FullValue(&|session| {
        let dur = session.duration();
        format!(
            "{:0}:{:0>2}.{:0>2}",
            dur.as_secs() / 60,
            dur.as_secs() % 60,
            dur.subsec_millis() / 10
        )
    }),
    Row::Empty,
    Row::LabelValue("LEVEL:", &|session| session.stats().level().to_string()),
    Row::LabelValue("LINES:", &|session| {
        session.stats().cleared_lines().to_string()
    }),
    Row::Empty,
    Row::LabelValue("TURN:", &|session| session.stats().turn().to_string()),
    Row::LabelValue("SINGLES:", &|session| {
        session.stats().line_cleared_counter()[1].to_string()
    }),
    Row::LabelValue("DOUBLES:", &|session| {
        session.stats().line_cleared_counter()[2].to_string()
    }),
    Row::LabelValue("TRIPLES:", &|session| {
        session.stats().line_cleared_counter()[3].to_string()
    }),
    Row::LabelValue("TETRIS:", &|session| {
        session.stats().line_cleared_counter()[4].to_string()
    }),
];

impl Widget for SessionStatsDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.as_ref().render(area, buf);
        let area = self.block.inner_if_some(area);

        let style = style::DEFAULT;

        let rows_areas =
            Layout::vertical((0..ROWS.len()).map(|_| Constraint::Length(1))).split(area);

        for (row, area) in iter::zip(ROWS.iter().copied(), rows_areas[..].iter().copied()) {
            match row {
                Row::Empty => {}
                Row::FullLabel(label) => {
                    Line::styled(label, style).left_aligned().render(area, buf);
                }
                Row::FullValue(value) => {
                    Line::styled(value(self.session), style)
                        .right_aligned()
                        .render(area, buf);
                }
                Row::LabelValue(label, value) => {
                    let [label_area, value_area] = area.layout(&Layout::horizontal([
                        Constraint::Fill(1),
                        Constraint::Fill(1),
                    ]));
                    Line::styled(label, style)
                        .left_aligned()
                        .render(label_area, buf);
                    Line::styled(value(self.session), style)
                        .right_aligned()
                        .render(value_area, buf);
                }
            }
        }
    }
}
