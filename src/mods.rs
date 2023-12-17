use std::path::PathBuf;

use anyhow::{Ok, Result as AnyResult};
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::*};

use crate::{
    app::{GlobalState, TabUi},
    colors::{BACKGROUND, SELECT},
    mods_views::ModView,
};

const MOD_DISPLAY_SIZE: usize = 5;
static GAME_PATH: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Titanfall2"#;

#[derive(Default)]
pub struct Mods {
    mod_view: ModView,
    page_offset: usize,
    selected_mod: usize,
    selected_view: usize,
}

impl TabUi for Mods {
    fn render_ui(&self, frame: &mut ratatui::Frame<'_>, rect: Rect) {
        let layout = Layout::new()
            .constraints([
                Constraint::Percentage(5),
                Constraint::Percentage(1),
                Constraint::Percentage(94),
            ])
            .direction(Direction::Horizontal)
            .split(rect);
        let mods_layout = Layout::new()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20); MOD_DISPLAY_SIZE])
            .split(layout[2]);

        let side_layout = Layout::new()
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(layout[0].height - 2 * 4),
            ])
            .split(layout[0]);
        ["Mods", "Plugins", "Packages", "cta"]
            .into_iter()
            .enumerate()
            .zip(side_layout.into_iter().cloned())
            .for_each(|((e, label), rect)| {
                frame.render_widget(
                    Paragraph::new(label)
                        .style(
                            e.eq(&self.selected_view)
                                .then(|| Style::default().red())
                                .unwrap_or_else(|| Style::default().white()),
                        )
                        .alignment(Alignment::Center)
                        .bg(BACKGROUND),
                    rect,
                )
            });
        frame.render_widget(Block::new().bg(BACKGROUND), side_layout[4]);

        frame.render_widget(
            Block::new()
                .bg(BACKGROUND)
                .borders(Borders::LEFT)
                .white()
                .title(self.mod_view.len().to_string()),
            layout[1],
        );

        mods_layout
            .into_iter()
            .cloned()
            .map(|vertical| {
                Layout::new()
                    .constraints([Constraint::Percentage(20); MOD_DISPLAY_SIZE])
                    .split(vertical)
            })
            .enumerate()
            .map(|(i, layout)| {
                layout
                    .into_iter()
                    .enumerate()
                    .map(move |(e, rect)| (e * MOD_DISPLAY_SIZE + i + self.page_offset, rect))
                    .map(|(e, rect)| {
                        self.mod_view
                            .get_as_paragraph(e)
                            .map(|paragraph| {
                                (
                                    Paragraph::new(paragraph)
                                        .bg(BACKGROUND)
                                        .wrap(Wrap { trim: true })
                                        .fg(SELECT)
                                        .alignment(Alignment::Center)
                                        .bold()
                                        .block(
                                            Block::default()
                                                .title(
                                                    self.mod_view
                                                        .get_title(e)
                                                        .unwrap_or_else(|| "UNK"),
                                                )
                                                .title_style(Style::default().light_red())
                                                .borders(Borders::all())
                                                .style(
                                                    self.selected_mod
                                                        .eq(&e)
                                                        .then(|| Style::default().red())
                                                        .unwrap_or_default(),
                                                ),
                                        ),
                                    rect.clone(),
                                )
                            })
                            .unwrap_or_else(|| {
                                (
                                    Paragraph::new("EMPTY")
                                        .bg(BACKGROUND)
                                        .alignment(Alignment::Center)
                                        .block(
                                            Block::default().borders(Borders::all()).style(
                                                self.selected_mod
                                                    .eq(&e)
                                                    .then(|| Style::default().red())
                                                    .unwrap_or_default(),
                                            ),
                                        ),
                                    rect.clone(),
                                )
                            })
                    })
                    .collect::<Vec<(Paragraph<'_>, Rect)>>()
            })
            .flatten()
            .for_each(|(text, rect)| frame.render_widget(text, rect));
    }

    fn render_binds(&self) -> Vec<&'static str> {
        vec![
            "reload - ctr + r",
            "next - n",
            "previous - p",
            "select - (↑/↓/→/←)/(h/j/k/l)",
            "type - tab",
        ]
    }

    fn update(&mut self, crosstermevent: event::Event, state: &mut GlobalState) -> AnyResult<()> {
        if let event::Event::Key(key_event) = crosstermevent {
            if key_event.kind == event::KeyEventKind::Press && !state.is_typing {
                match key_event.code {
                    KeyCode::Char('r') if key_event.modifiers == KeyModifiers::CONTROL => {
                        self.mod_view.reload(&PathBuf::from(GAME_PATH))?;
                    }
                    KeyCode::Char('n') => {
                        self.page_offset = self
                            .mod_view
                            .len()
                            .div_ceil(MOD_DISPLAY_SIZE.pow(2))
                            .saturating_sub(1)
                            .saturating_mul(MOD_DISPLAY_SIZE.pow(2))
                            .min(self.page_offset + MOD_DISPLAY_SIZE.pow(2));
                    }
                    KeyCode::Char('p') => {
                        self.page_offset = self.page_offset.saturating_sub(MOD_DISPLAY_SIZE.pow(2));
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.selected_mod =
                            self.page_offset.max(self.selected_mod.saturating_sub(1))
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.selected_mod = self
                            .page_offset
                            .saturating_add(MOD_DISPLAY_SIZE.pow(2) - 1)
                            .min(self.selected_mod + 1)
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.selected_mod = self
                            .page_offset
                            .saturating_add(MOD_DISPLAY_SIZE.pow(2) - 1)
                            .min(self.selected_mod + MOD_DISPLAY_SIZE)
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.selected_mod = self
                            .page_offset
                            .max(self.selected_mod.saturating_sub(MOD_DISPLAY_SIZE))
                    }
                    KeyCode::Tab => {
                        self.selected_view = self
                            .selected_view
                            .eq(&3) // there are only 4 views
                            .then(|| 0)
                            .unwrap_or_else(|| self.selected_view + 1);
                        (self.page_offset, self.selected_mod) = (0, 0);
                        self.mod_view
                            .switch(self.selected_view)?
                            .reload(&PathBuf::from(GAME_PATH))?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn on_focus(&mut self) -> AnyResult<()> {
        // direct path for now
        // TODO: add path search
        self.mod_view.reload(&PathBuf::from(GAME_PATH))?;

        Ok(())
    }
}
