use anyhow::Ok;
use crossterm::event::{self, KeyCode};
use once_cell::sync::Lazy;
use ratatui::{prelude::*, widgets::*};

use crate::{
    app::{GlobalState, TabUi},
    colors::BACKGROUND,
};

static NAME_PLATE: Lazy<Vec<String>> = Lazy::new(|| {
    include_str!("..\\name_plate.txt")
        .split("DIRECTIVE_SPLIT")
        .map(str::to_string)
        .collect::<Vec<String>>()
});

#[derive(Default)]
pub struct MainMenu {
    playing: bool,
}

impl TabUi for MainMenu {
    fn render_ui(&self, frame: &mut Frame<'_>, rect: Rect) {
        let layout = Layout::new()
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(rect);

        let name_plate = Paragraph::new(NAME_PLATE[0].clone())
            .bg(BACKGROUND)
            .red()
            .add_modifier(Modifier::BOLD);

        frame.render_widget(name_plate, layout[0]);

        frame.render_widget(
            Block::new().borders(Borders::empty()).bg(BACKGROUND),
            layout[1],
        );

        let button = Paragraph::new(
            self.playing
                .then(|| NAME_PLATE[1].clone())
                .unwrap_or_else(|| NAME_PLATE[2].clone()),
        )
        .block(
            Block::default()
                .borders(Borders::all())
                .padding(Padding::zero())
                .add_modifier(Modifier::BOLD)
                .black()
                .bg(BACKGROUND),
        )
        .style(
            self.playing
                .then(|| Style::default().light_red())
                .unwrap_or_else(|| Style::default()),
        )
        .add_modifier(Modifier::BOLD)
        .bg(BACKGROUND);

        frame.render_widget(button, layout[2]);
    }

    fn render_binds(&self) -> Vec<&'static str> {
        vec!["play - p"]
    }

    fn update(
        &mut self,
        crosstermevent: event::Event,
        state: &mut GlobalState,
    ) -> anyhow::Result<()> {
        if let event::Event::Key(key_event) = crosstermevent {
            if key_event.kind == event::KeyEventKind::Press && !state.is_typing {
                match key_event.code {
                    KeyCode::Char('p') => self.playing = !self.playing,
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn on_focus(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
