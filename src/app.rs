use std::time::{Duration, SystemTime};

use anyhow::{Ok, Result as AnyResult};
use crossterm::event::{self, Event::Key, KeyCode::Char, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::{
    colors::{BACKGROUND, VBLACK, VGREY},
    mainmenu::MainMenu,
    mods::Mods,
};

pub trait TabUi {
    fn render_ui(&self, frame: &mut Frame<'_>, rect: Rect);
    fn render_binds(&self) -> Vec<&'static str>;
    fn update(&mut self, crosstermevent: event::Event, state: &mut GlobalState) -> AnyResult<()>;
    fn on_focus(&mut self) -> AnyResult<()>;
}

pub struct GlobalState {
    pub is_typing: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Tab {
    #[default]
    Main,
    Mods,
    Utils,
    Max, // doesn't have a label
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Tab::Main => "main",
            Tab::Mods => "mods",
            Tab::Utils => "utils",
            Tab::Max => unreachable!("should be called on this"),
        }
        .to_uppercase();

        write!(f, "{}({})", display, *self as u32 + 1)
    }
}

impl Into<u32> for Tab {
    fn into(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for Tab {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if Tab::Max as u32 <= value {
            Err(value)
        } else {
            Result::Ok(unsafe { std::mem::transmute(value) })
        }
    }
}

pub struct App {
    pub should_quit: bool,
    tab: Tab,
    tabs: Vec<Box<dyn TabUi>>,
    last_error: Option<(SystemTime, anyhow::Error)>,
    state: GlobalState,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            tab: Tab::default(),
            tabs: vec![Box::new(MainMenu::default()), Box::new(Mods::default())],
            last_error: None,
            state: GlobalState { is_typing: false },
        }
    }

    pub fn run(&mut self) -> AnyResult<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

        while !self.should_quit {
            self.update()?;

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let max_size = frame.size();
        let draw_size = frame.size().height - 3;
        let layout = Layout::new()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(draw_size),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(max_size);
        frame.render_widget(
            Tabs::new(
                vec![Tab::Main, Tab::Mods, Tab::Utils]
                    .into_iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<String>>(),
            )
            .block(Block::default().borders(Borders::NONE))
            .style(Style::new().bg(VBLACK).fg(VGREY))
            .highlight_style(Style::default().red().bg(VBLACK))
            .select(self.tab as usize)
            .divider(" | "),
            layout[0],
        );

        self.tabs
            .get(self.tab as usize)
            .expect("label not implemented for implemented tab ui wtf?!")
            .render_ui(frame, layout[1]);

        let mut binds = vec!["Tabs - (1-9)", "Quit - q"];
        binds.extend(
            self.tabs
                .get(self.tab as usize)
                .expect("tab should ready")
                .render_binds(),
        );
        frame.render_widget(
            Tabs::new(
                binds
                    .into_iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<String>>(),
            )
            .block(Block::default().borders(Borders::NONE))
            .style(Style::new().bg(VBLACK))
            .divider("  "),
            layout[2],
        );
        frame.render_widget(
            Paragraph::new(
                self.last_error
                    .as_ref()
                    .map(|err| err.1.to_string())
                    .unwrap_or_else(|| "".to_owned()),
            )
            .style(Style::default().bg(BACKGROUND))
            .red(),
            layout[3],
        );
    }

    pub fn update(&mut self) -> AnyResult<()> {
        if event::poll(Duration::from_millis(10))? {
            let crosstermevent = event::read()?;
            if let Key(key) = crosstermevent {
                if key.kind == event::KeyEventKind::Press && !self.state.is_typing {
                    match key.code {
                        Char(keycode) => {
                            let tab = keycode as u8 - b'1';
                            self.tabs.get(tab as usize).map(|_| {
                                (tab as u32)
                                    .try_into()
                                    .expect("label not implemented for implemented tab ui wtf?!")
                            })
                        }
                        _ => None,
                    }
                    .map(|t| self.switch_tabs(t));

                    match key.code {
                        Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            self.should_quit = true
                        }
                        Char('q') | event::KeyCode::Esc => self.should_quit = true,
                        _ => {}
                    }
                }
            }

            if let Err(err) = self
                .tabs
                .get_mut(self.tab as usize)
                .expect("label not implemented for implemented tab ui wtf?!")
                .update(crosstermevent, &mut self.state)
            {
                self.last_error = Some((SystemTime::now() + Duration::from_secs(2), err))
            }

            if self
                .last_error
                .as_ref()
                .is_some_and(|last_error| last_error.0 < SystemTime::now())
            {
                self.last_error = None;
            }
        };

        Ok(())
    }

    fn switch_tabs(&mut self, new_tab: Tab) {
        self.tab = new_tab;
        self.state.is_typing = false;

        if let Err(err) = self
            .tabs
            .get_mut(self.tab as usize)
            .expect("label not implemented for implemented tab ui wtf?!")
            .on_focus()
        {
            self.last_error = Some((SystemTime::now() + Duration::from_secs(2), err))
        }
    }
}
