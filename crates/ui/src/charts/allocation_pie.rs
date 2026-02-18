use iced::widget::canvas::{self, Canvas, Geometry, Path, Text};
use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};
use std::f32::consts::PI;

use retirement_planner_core::models::Instrument;

const SLICE_COLORS: &[Color] = &[
    Color::from_rgb(0.25, 0.50, 0.90),
    Color::from_rgb(0.90, 0.40, 0.15),
    Color::from_rgb(0.20, 0.72, 0.40),
    Color::from_rgb(0.80, 0.20, 0.60),
    Color::from_rgb(0.95, 0.65, 0.15),
    Color::from_rgb(0.40, 0.75, 0.85),
    Color::from_rgb(0.60, 0.40, 0.80),
];

#[derive(Debug, Clone, Default)]
pub struct AllocationPieChart {
    pub slices: Vec<(String, i64)>,
    pub title: String,
}

impl AllocationPieChart {
    pub fn from_instruments(instruments: &[Instrument], title: String) -> Self {
        let mut groups: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
        for inst in instruments {
            *groups
                .entry(inst.asset_class.as_str().to_string())
                .or_default() += inst.current_value_cents;
        }

        let slices: Vec<(String, i64)> = groups
            .into_iter()
            .filter(|(_, v)| *v > 0)
            .collect();

        Self {
            slices,
            title,
        }
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(250)
            .into()
    }
}

impl canvas::Program<()> for AllocationPieChart {
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

            if self.slices.is_empty() {
                frame.fill_text(Text {
                    content: "No allocations to display".to_string(),
                    position: Point::new(w / 2.0, h / 2.0),
                    size: 14.0.into(),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    ..Text::default()
                });
                return vec![frame.into_geometry()];
            }

            // Title
            frame.fill_text(Text {
                content: self.title.clone(),
                position: Point::new(w / 2.0, 10.0),
                size: 15.0.into(),
                color: Color::from_rgb(0.15, 0.15, 0.2),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                ..Text::default()
            });

            let total: i64 = self.slices.iter().map(|(_, v)| v).sum();
            if total == 0 {
                return vec![frame.into_geometry()];
            }

            let cx = w * 0.35;
            let cy = h / 2.0 + 10.0;
            let radius = (w.min(h) * 0.32).min(100.0);

            // Draw pie slices using filled triangles approximation
            let mut start_angle: f32 = -PI / 2.0;
            for (i, (name, value)) in self.slices.iter().enumerate() {
                let frac = *value as f32 / total as f32;
                let sweep = frac * 2.0 * PI;
                let color = SLICE_COLORS[i % SLICE_COLORS.len()];

                // Draw arc as filled wedge using small line segments
                let segments = (sweep / 0.05).ceil() as usize;
                let segments = segments.max(2);

                let wedge = Path::new(|builder| {
                    builder.move_to(Point::new(cx, cy));
                    for s in 0..=segments {
                        let angle = start_angle + sweep * (s as f32 / segments as f32);
                        let px = cx + radius * angle.cos();
                        let py = cy + radius * angle.sin();
                        builder.line_to(Point::new(px, py));
                    }
                    builder.line_to(Point::new(cx, cy));
                });

                frame.fill(&wedge, color);

                // Legend (right side)
                let legend_x = w * 0.62;
                let legend_y = 40.0 + i as f32 * 22.0;

                frame.fill_rectangle(
                    Point::new(legend_x, legend_y),
                    Size::new(14.0, 14.0),
                    color,
                );

                let pct = frac * 100.0;
                frame.fill_text(Text {
                    content: format!("{} ({:.0}%)", name, pct),
                    position: Point::new(legend_x + 20.0, legend_y),
                    size: 12.0.into(),
                    color: Color::from_rgb(0.2, 0.2, 0.25),
                    ..Text::default()
                });

                start_angle += sweep;
            }
        }

        vec![frame.into_geometry()]
    }
}
