use crossterm::event::{Event, KeyCode};
use oxidris_analysis::sample::BoardSample;
use oxidris_evaluator::board_feature::BoxedBoardFeature;
use oxidris_stats::comprehensive::ComprehensiveStats;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Spacing},
    prelude::Direction,
    style::{Color, Modifier, Style},
    symbols::{Marker, merge::MergeStrategy},
    text::{Line, Text},
    widgets::{
        Axis, Bar, BarChart, Block, Chart, Dataset, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget,
    },
};

use crate::command::analyze_board_features::app::AppData;

#[derive(Debug)]
pub struct FeatureListScreen {
    data: AppData,
    features: Vec<BoxedBoardFeature>,
    selected_feature: usize,
    clip_scatter_plot: bool,
    should_exit: bool,
}

impl FeatureListScreen {
    #[must_use]
    pub fn new(data: AppData, features: Vec<BoxedBoardFeature>) -> Self {
        Self {
            data,
            features,
            selected_feature: 0,
            clip_scatter_plot: true,
            should_exit: false,
        }
    }

    pub(crate) fn should_exit(&self) -> bool {
        self.should_exit
    }

    /// Extract raw feature values and pair them with a target value (transformed or normalized).
    #[expect(clippy::cast_precision_loss)]
    fn build_scatter_data<F>(&self, extract_y: F) -> Vec<(f64, f64)>
    where
        F: Fn(&BoardSample) -> f32,
    {
        self.data
            .board_samples
            .iter()
            .map(|sample| {
                let raw = f64::from(sample.feature_vector[self.selected_feature].raw as f32);
                let y = f64::from(extract_y(sample));
                (raw, y)
            })
            .collect()
    }

    /// Calculate the Y-axis range for data points within the clipped X range.
    fn calculate_y_range_for_clipped_x(data: &[(f64, f64)], x_max: f64) -> [f64; 2] {
        let y_max = data
            .iter()
            .copied()
            .filter_map(|(x, y)| (x <= x_max).then_some(y))
            .max_by(f64::total_cmp)
            .unwrap_or(0.0);
        [0.0, y_max]
    }

    pub fn draw(&self, frame: &mut Frame) {
        let feature_stats = &self.data.statistics[self.selected_feature];

        // Layout: main area + help line at bottom
        let [main_area, help_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .spacing(Spacing::Overlap(1))
                .areas(main_area);

        let [feature_pane, trans_chart_pane, norm_chart_pane] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(Spacing::Overlap(1))
        .areas(left_area);

        let [
            raw_stats_pane,
            transformed_stats_pane,
            normalized_stats_pane,
        ] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .spacing(Spacing::Overlap(1))
        .areas(right_area);

        // Calculate X-axis range (clipped or full)
        let raw_max = if self.clip_scatter_plot {
            feature_stats
                .raw
                .percentiles
                .get(95.0)
                .unwrap_or(feature_stats.raw.stats.max)
        } else {
            feature_stats.raw.stats.max
        };
        let raw_range = [0.0, f64::from(raw_max)];

        // Build scatter plot data
        let raw_trans_data = self
            .build_scatter_data(|sample| sample.feature_vector[self.selected_feature].transformed);
        let raw_norm_data = self
            .build_scatter_data(|sample| sample.feature_vector[self.selected_feature].normalized);

        // Calculate Y-axis range for transformed values
        let trans_range = Self::calculate_y_range_for_clipped_x(&raw_trans_data, raw_range[1]);

        // Create widgets
        let feature_list_widget = FeatureSelector {
            features: &self.features,
            selected_feature: self.selected_feature,
        };

        let raw_trans_chart = FeatureScatter {
            label: "Raw vs Transformed",
            data: &raw_trans_data,
            x_title: "Raw",
            x_bounds: raw_range,
            y_title: "Transformed",
            y_bounds: trans_range,
        };

        let raw_norm_chart = FeatureScatter {
            label: "Raw vs Normalized",
            data: &raw_norm_data,
            x_title: "Raw",
            x_bounds: raw_range,
            y_title: "Normalized",
            y_bounds: [0.0, 1.0],
        };

        let raw_stats_widget = FeatureStatistics {
            label: "Raw",
            stats: &feature_stats.raw,
        };
        let transformed_stats_widget = FeatureStatistics {
            label: "Transformed",
            stats: &feature_stats.transformed,
        };
        let normalized_stats_widget = FeatureStatistics {
            label: "Normalized",
            stats: &feature_stats.normalized,
        };

        // Render left panes: feature list and scatter plots
        frame.render_widget(feature_list_widget, feature_pane);
        frame.render_widget(raw_trans_chart, trans_chart_pane);
        frame.render_widget(raw_norm_chart, norm_chart_pane);

        // Render right panes: statistics for each transformation stage
        frame.render_widget(raw_stats_widget, raw_stats_pane);
        frame.render_widget(transformed_stats_widget, transformed_stats_pane);
        frame.render_widget(normalized_stats_widget, normalized_stats_pane);

        // Render help line
        let clip_status = if self.clip_scatter_plot { "P95" } else { "Max" };
        let help_text = Text::from(format!(
            "↑/↓: Select | c: Toggle Clip ({clip_status}) | q/Esc: Quit"
        ))
        .style(Style::default().fg(Color::DarkGray))
        .centered();
        frame.render_widget(help_text, help_area);
    }

    pub(crate) fn handle_event(&mut self, event: &Event) {
        if let Some(event) = event.as_key_event() {
            match event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                KeyCode::Char('c') => {
                    self.clip_scatter_plot = !self.clip_scatter_plot;
                }
                KeyCode::Up if !self.features.is_empty() => {
                    self.selected_feature = self
                        .selected_feature
                        .checked_sub(1)
                        .unwrap_or(self.features.len() - 1);
                }
                KeyCode::Down if !self.features.is_empty() => {
                    self.selected_feature = (self.selected_feature + 1) % self.features.len();
                }
                _ => {}
            }
        }
    }
}

struct FeatureSelector<'a> {
    features: &'a [BoxedBoardFeature],
    selected_feature: usize,
}

impl Widget for FeatureSelector<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let items = self
            .features
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let content = format!("{i}: {}", f.name());
                ListItem::new(content)
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Features")
                    .merge_borders(MergeStrategy::Exact),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_feature));

        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}

struct FeatureStatistics<'a> {
    label: &'a str,
    stats: &'a ComprehensiveStats,
}

impl Widget for FeatureStatistics<'_> {
    #[expect(clippy::cast_precision_loss)]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .merge_borders(MergeStrategy::Exact)
            .title(self.label);

        let [stats_area, chart_area] =
            Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)])
                .areas(block.inner(area));

        let stats = &self.stats.stats;
        let mut text = vec![
            Line::raw(format!("  Mean:   {:10.2}", stats.mean)),
            Line::raw(format!("  Median: {:10.2}", stats.median)),
            Line::raw(format!("  Min:    {:10.2}", stats.min,)),
        ];
        for p in [1, 5, 10, 25, 50, 75, 90, 95, 99] {
            text.push(Line::raw(format!(
                "  P{p:02}:    {:10.2}",
                self.stats.percentiles.get(p as f32).unwrap_or(f32::NAN)
            )));
        }
        text.extend([
            Line::raw(format!("  Max:    {:10.2}", stats.max,)),
            Line::raw(format!("  StdDev: {:10.2}", stats.std_dev,)),
        ]);

        let paragraph = Paragraph::new(text);
        let chart = BarChart::new(
            self.stats
                .histogram
                .bins
                .iter()
                .map(|bin| {
                    Bar::with_label(
                        format!("{:8.2}-{:8.2}", bin.range.start, bin.range.end),
                        bin.count,
                    )
                    .text_value(format!("{}", bin.count))
                })
                .collect::<Vec<_>>(),
        )
        .direction(Direction::Horizontal)
        .bar_gap(0);

        Widget::render(block, area, buf);
        Widget::render(paragraph, stats_area, buf);
        Widget::render(chart, chart_area, buf);
    }
}

struct FeatureScatter<'a> {
    label: &'a str,
    data: &'a [(f64, f64)],
    x_title: &'a str,
    x_bounds: [f64; 2],
    y_title: &'a str,
    y_bounds: [f64; 2],
}

impl Widget for FeatureScatter<'_> {
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
