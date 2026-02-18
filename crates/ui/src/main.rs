//! Retirement Planner UI
//!
//! iced-based GUI for the retirement planning application.

mod app;
mod charts;
mod theme;
mod views;

use app::App;

pub fn main() -> iced::Result {
    iced::application("Retirement Planner", App::update, App::view)
        .window_size((1200.0, 800.0))
        .run()
}
