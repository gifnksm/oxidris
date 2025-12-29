use crossterm::event::{KeyCode, KeyEvent};
use oxidris_ai::ALL_METRICS;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Spacing},
    prelude::Direction,
    style::{Color, Modifier, Style},
    symbols::{Marker, merge::MergeStrategy},
    text::Line,
    widgets::{
        Axis, Bar, BarChart, Block, Chart, Dataset, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget,
    },
};

use crate::tune_metrics::{
    data::ValueStats,
    ui::app::{AppData, Screen},
};

#[derive(Default, Debug)]
pub struct MetricsListScreen {
    selected_metric: usize,
}

impl MetricsListScreen {
    #[expect(clippy::cast_precision_loss)]
    pub fn draw(&self, frame: &mut Frame, data: &AppData) {
        let metric_stats = &data.statistics[self.selected_metric];

        let panes = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
            .spacing(Spacing::Overlap(1))
            .split(frame.area());

        let left_panes = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(Spacing::Overlap(1))
        .split(panes[0]);

        let right_panes = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(Spacing::Overlap(1))
        .split(panes[1]);

        let metric_list_pane = MetricSelector {
            selected_metric: self.selected_metric,
        };

        let raw_pane = MetricStatistics {
            label: "Raw",
            stats: &metric_stats.raw,
        };
        let transformed_pane = MetricStatistics {
            label: "Transformed",
            stats: &metric_stats.transformed,
        };
        let normalized_pane = MetricStatistics {
            label: "Normalized",
            stats: &metric_stats.normalized,
        };

        let raw_trans_data = data
            .boards_metrics
            .iter()
            .map(|bm| {
                (
                    f64::from(bm.metrics[self.selected_metric].raw as f32),
                    f64::from(bm.metrics[self.selected_metric].transformed),
                )
            })
            .collect::<Vec<(f64, f64)>>();

        let raw_trans_chart = MetricScatter {
            label: "Raw vs Transformed",
            data: &raw_trans_data,
            x_title: "Raw",
            x_bounds: [0.0, f64::from(metric_stats.raw.max)],
            y_title: "Transformed",
            y_bounds: [0.0, f64::from(metric_stats.transformed.max)],
        };

        let raw_norm_data = data
            .boards_metrics
            .iter()
            .map(|bm| {
                (
                    f64::from(bm.metrics[self.selected_metric].raw as f32),
                    f64::from(bm.metrics[self.selected_metric].normalized),
                )
            })
            .collect::<Vec<(f64, f64)>>();
        let raw_norm_chart = MetricScatter {
            label: "Raw vs Normalized",
            data: &raw_norm_data,
            x_title: "Raw",
            x_bounds: [0.0, f64::from(metric_stats.raw.max)],
            y_title: "Normalized",
            y_bounds: [0.0, 1.0],
        };

        frame.render_widget(metric_list_pane, left_panes[0]);
        frame.render_widget(raw_trans_chart, left_panes[1]);
        frame.render_widget(raw_norm_chart, left_panes[2]);

        frame.render_widget(raw_pane, right_panes[0]);
        frame.render_widget(transformed_pane, right_panes[1]);
        frame.render_widget(normalized_pane, right_panes[2]);
    }

    pub(crate) fn handle_input(&mut self, key_event: KeyEvent, screen: &mut Screen) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                *screen = Screen::Exiting;
            }
            KeyCode::Up => {
                if self.selected_metric == 0 {
                    self.selected_metric = ALL_METRICS.len() - 1;
                } else {
                    self.selected_metric -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_metric + 1 >= ALL_METRICS.len() {
                    self.selected_metric = 0;
                } else {
                    self.selected_metric += 1;
                }
            }
            _ => {}
        }
    }
}

struct MetricSelector {
    selected_metric: usize,
}

impl Widget for MetricSelector {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let items = ALL_METRICS
            .as_array()
            .iter()
            .enumerate()
            .map(|(i, metric)| {
                let content = format!("{i}: {}", metric.name());
                ListItem::new(content)
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Metrics")
                    .merge_borders(MergeStrategy::Exact),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_metric));

        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}

struct MetricStatistics<'a> {
    label: &'a str,
    stats: &'a ValueStats,
}

impl Widget for MetricStatistics<'_> {
    #[expect(clippy::cast_precision_loss)]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .merge_borders(MergeStrategy::Exact)
            .title(self.label);

        let panes = Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)])
            .split(block.inner(area));

        let mut text = vec![
            Line::raw(format!("  Mean:   {:10.2}", self.stats.mean)),
            Line::raw(format!("  Median: {:10.2}", self.stats.median)),
            Line::raw(format!("  Min:    {:10.2}", self.stats.min,)),
        ];
        for p in [10, 25, 75, 90, 95, 99] {
            text.push(Line::raw(format!(
                "  P{p:02}:    {:10.2}",
                self.stats.get_percentile(p as f32).unwrap_or(f32::NAN)
            )));
        }
        text.extend([
            Line::raw(format!("  Max:    {:10.2}", self.stats.max,)),
            Line::raw(format!("  StdDev: {:10.2}", self.stats.std_dev,)),
        ]);

        let paragraph = Paragraph::new(text);
        let chart = BarChart::new(
            self.stats
                .histogram
                .bins
                .iter()
                .map(|bin| {
                    {
                        Bar::with_label(
                            format!("{:8.2}-{:8.2}", bin.range.start, bin.range.end),
                            bin.count,
                        )
                        .text_value(format!("{}", bin.count))
                    }
                })
                .collect::<Vec<_>>(),
        )
        .direction(Direction::Horizontal)
        .bar_gap(0);

        Widget::render(block, area, buf);
        Widget::render(paragraph, panes[0], buf);
        Widget::render(chart, panes[1], buf);
    }
}

struct MetricScatter<'a> {
    label: &'a str,
    data: &'a [(f64, f64)],
    x_title: &'a str,
    x_bounds: [f64; 2],
    y_title: &'a str,
    y_bounds: [f64; 2],
}

impl Widget for MetricScatter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let dataset = Dataset::default()
            .marker(Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(self.data);
        let x_axis = Axis::default()
            .title(self.x_title)
            .bounds(self.x_bounds)
            .labels([
                format!("{:.2}", self.x_bounds[0]),
                format!("{:.2}", f64::midpoint(self.x_bounds[0], self.x_bounds[1])),
                format!("{:.2}", self.x_bounds[1]),
            ]);
        let y_axis = Axis::default()
            .title(self.y_title)
            .bounds(self.y_bounds)
            .labels([
                format!("{:.2}", self.y_bounds[0]),
                format!("{:.2}", f64::midpoint(self.y_bounds[0], self.y_bounds[1])),
                format!("{:.2}", self.y_bounds[1]),
            ]);
        let chart = Chart::new(vec![dataset])
            .block(
                Block::bordered()
                    .merge_borders(MergeStrategy::Exact)
                    .title(self.label),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        Widget::render(chart, area, buf);
    }
}
