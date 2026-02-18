use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};

use retirement_planner_core::engine::YearProjection;

/// Colors for different scenario lines
const SCENARIO_COLORS: &[Color] = &[
    Color::from_rgb(0.25, 0.50, 0.90),  // blue
    Color::from_rgb(0.90, 0.40, 0.15),  // orange
    Color::from_rgb(0.20, 0.72, 0.40),  // green
    Color::from_rgb(0.80, 0.20, 0.60),  // purple
    Color::from_rgb(0.95, 0.65, 0.15),  // amber
];

#[derive(Debug, Clone, Default)]
pub struct ScenarioLine {
    pub name: String,
    pub projections: Vec<YearProjection>,
}

#[derive(Debug, Clone, Default)]
pub struct CorpusLineChart {
    pub scenarios: Vec<ScenarioLine>,
    pub retirement_age: u8,
}

impl CorpusLineChart {
    pub fn new(scenarios: Vec<ScenarioLine>, retirement_age: u8) -> Self {
        Self {
            scenarios,
            retirement_age,
        }
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(350)
            .into()
    }
}

impl canvas::Program<()> for CorpusLineChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = iced::widget::canvas::Frame::new(renderer, bounds.size());
        {
            let w = bounds.width;
            let h = bounds.height;
            let margin_left = 80.0;
            let margin_right = 20.0;
            let margin_top = 40.0;
            let margin_bottom = 50.0;
            let chart_w = w - margin_left - margin_right;
            let chart_h = h - margin_top - margin_bottom;

            if self.scenarios.is_empty() || chart_w <= 0.0 || chart_h <= 0.0 {
                return vec![frame.into_geometry()];
            }

            // Find global min/max
            let mut min_age = u8::MAX;
            let mut max_age = 0u8;
            let mut max_corpus: i64 = 0;

            for s in &self.scenarios {
                for p in &s.projections {
                    min_age = min_age.min(p.age);
                    max_age = max_age.max(p.age);
                    max_corpus = max_corpus.max(p.corpus_cents);
                }
            }

            if max_corpus == 0 || max_age <= min_age {
                return vec![frame.into_geometry()];
            }

            let age_range = (max_age - min_age) as f32;
            let corpus_max = max_corpus as f64;

            // Background
            frame.fill_rectangle(
                Point::new(margin_left, margin_top),
                Size::new(chart_w, chart_h),
                Color::from_rgb(0.98, 0.98, 0.99),
            );

            // Grid lines & Y-axis labels
            let grid_lines = 5;
            for i in 0..=grid_lines {
                let frac = i as f32 / grid_lines as f32;
                let y = margin_top + chart_h * (1.0 - frac);

                // Grid line
                let grid = Path::line(
                    Point::new(margin_left, y),
                    Point::new(margin_left + chart_w, y),
                );
                frame.stroke(
                    &grid,
                    Stroke::default()
                        .with_color(Color::from_rgb(0.88, 0.88, 0.90))
                        .with_width(1.0),
                );

                // Y label
                let value = corpus_max * frac as f64 / 100.0; // cents to dollars
                let label = if value >= 1_000_000.0 {
                    format!("{:.1}M", value / 1_000_000.0)
                } else if value >= 1_000.0 {
                    format!("{:.0}K", value / 1_000.0)
                } else {
                    format!("{:.0}", value)
                };

                frame.fill_text(Text {
                    content: label,
                    position: Point::new(margin_left - 8.0, y - 6.0),
                    size: 11.0.into(),
                    color: Color::from_rgb(0.4, 0.4, 0.5),
                    horizontal_alignment: iced::alignment::Horizontal::Right,
                    ..Text::default()
                });
            }

            // X-axis labels
            let x_step = if age_range > 30.0 { 10 } else { 5 };
            let mut age = min_age;
            while age <= max_age {
                let frac = (age - min_age) as f32 / age_range;
                let x = margin_left + chart_w * frac;

                // Tick
                let tick = Path::line(
                    Point::new(x, margin_top + chart_h),
                    Point::new(x, margin_top + chart_h + 5.0),
                );
                frame.stroke(
                    &tick,
                    Stroke::default()
                        .with_color(Color::from_rgb(0.5, 0.5, 0.5))
                        .with_width(1.0),
                );

                frame.fill_text(Text {
                    content: format!("{}", age),
                    position: Point::new(x, margin_top + chart_h + 8.0),
                    size: 11.0.into(),
                    color: Color::from_rgb(0.4, 0.4, 0.5),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    ..Text::default()
                });

                age = age.saturating_add(x_step);
            }

            // Retirement age vertical line
            if self.retirement_age >= min_age && self.retirement_age <= max_age {
                let frac = (self.retirement_age - min_age) as f32 / age_range;
                let x = margin_left + chart_w * frac;

                let retire_line = Path::line(
                    Point::new(x, margin_top),
                    Point::new(x, margin_top + chart_h),
                );
                frame.stroke(
                    &retire_line,
                    Stroke::default()
                        .with_color(Color::from_rgba(0.9, 0.25, 0.25, 0.5))
                        .with_width(1.5),
                );

                frame.fill_text(Text {
                    content: "Retire".to_string(),
                    position: Point::new(x + 4.0, margin_top + 2.0),
                    size: 10.0.into(),
                    color: Color::from_rgb(0.9, 0.25, 0.25),
                    ..Text::default()
                });
            }

            // Draw scenario lines
            for (si, scenario) in self.scenarios.iter().enumerate() {
                let color = SCENARIO_COLORS[si % SCENARIO_COLORS.len()];

                let points: Vec<Point> = scenario
                    .projections
                    .iter()
                    .map(|p| {
                        let x_frac = (p.age - min_age) as f32 / age_range;
                        let y_frac = p.corpus_cents as f64 / corpus_max;
                        Point::new(
                            margin_left + chart_w * x_frac,
                            margin_top + chart_h * (1.0 - y_frac as f32),
                        )
                    })
                    .collect();

                if points.len() >= 2 {
                    let line = Path::new(|builder| {
                        builder.move_to(points[0]);
                        for p in &points[1..] {
                            builder.line_to(*p);
                        }
                    });

                    frame.stroke(
                        &line,
                        Stroke::default().with_color(color).with_width(2.5),
                    );
                }

                // Legend entry
                let legend_y = margin_top + 4.0 + si as f32 * 18.0;
                let legend_x = margin_left + 10.0;
                frame.fill_rectangle(
                    Point::new(legend_x, legend_y),
                    Size::new(14.0, 3.0),
                    color,
                );
                frame.fill_text(Text {
                    content: scenario.name.clone(),
                    position: Point::new(legend_x + 18.0, legend_y - 4.0),
                    size: 11.0.into(),
                    color: Color::from_rgb(0.2, 0.2, 0.25),
                    ..Text::default()
                });
            }

            // Title
            frame.fill_text(Text {
                content: "Corpus Over Time".to_string(),
                position: Point::new(w / 2.0, 10.0),
                size: 15.0.into(),
                color: Color::from_rgb(0.15, 0.15, 0.2),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                ..Text::default()
            });

            // Axis labels
            frame.fill_text(Text {
                content: "Age".to_string(),
                position: Point::new(w / 2.0, h - 5.0),
                size: 12.0.into(),
                color: Color::from_rgb(0.4, 0.4, 0.5),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                ..Text::default()
            });

            // Axes
            let axes = Path::new(|builder| {
                builder.move_to(Point::new(margin_left, margin_top));
                builder.line_to(Point::new(margin_left, margin_top + chart_h));
                builder.line_to(Point::new(margin_left + chart_w, margin_top + chart_h));
            });
            frame.stroke(
                &axes,
                Stroke::default()
                    .with_color(Color::from_rgb(0.5, 0.5, 0.55))
                    .with_width(1.5),
            );
        }

        vec![frame.into_geometry()]
    }
}
