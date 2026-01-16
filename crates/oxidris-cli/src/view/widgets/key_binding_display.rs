use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block as BlockWidget, BlockExt, Widget},
};

pub type KeyBinding<'a> = (&'a [&'a str], &'a str);

#[derive(Debug)]
pub struct KeyBindingDisplay<'a> {
    bindings: &'a [KeyBinding<'a>],
    block: Option<BlockWidget<'a>>,
}

impl<'a> KeyBindingDisplay<'a> {
    pub fn new(bindings: &'a [KeyBinding<'a>]) -> Self {
        Self {
            bindings,
            block: None,
        }
    }

    pub fn block(self, block: BlockWidget<'a>) -> Self {
        Self {
            block: Some(block),
            ..self
        }
    }
}

const KEY_STYLE: Style = Style::new().fg(Color::Cyan);
const KEY_SEPARATOR_STYLE: Style = Style::new().fg(Color::DarkGray);
const DESCRIPTION_STYLE: Style = Style::new().fg(Color::White);
const ITEM_SEPARATOR_STYLE: Style = Style::new().fg(Color::DarkGray);

impl Widget for KeyBindingDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.block.as_ref().render(area, buf);
        let area = self.block.inner_if_some(area);

        let mut spans = vec![];

        for (i, (keys, desc)) in self.bindings.iter().copied().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" | ", ITEM_SEPARATOR_STYLE));
            }
            for (i, key) in keys.iter().copied().enumerate() {
                if i > 0 {
                    spans.push(Span::styled("/", KEY_SEPARATOR_STYLE));
                }
                spans.push(Span::styled(key, KEY_STYLE));
            }
            spans.push(Span::from(" "));
            spans.push(Span::styled(desc, DESCRIPTION_STYLE));
        }

        let text = Line::from(spans).centered();
        text.render(area, buf);
    }
}
