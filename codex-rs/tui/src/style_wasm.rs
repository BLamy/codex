use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;

pub fn user_message_style() -> Style {
    Style::default().bg(Color::Rgb(39, 41, 53))
}

pub(crate) fn accent_style() -> Style {
    Style::default().fg(Color::Cyan).bold()
}

pub(crate) fn table_separator_style() -> Style {
    Style::default().dim()
}
