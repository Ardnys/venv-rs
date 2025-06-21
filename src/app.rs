use std::path::Path;

use crate::{
    event::{AppEvent, Event, EventHandler},
    venv::{Venv, VenvList},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    /// Event handler.
    pub events: EventHandler,
    pub venv_list: VenvList,
    pub venv_index: usize,
    // TODO: selected venv
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            venv_index: 0,
            venv_list: VenvList::from_iter([
                (
                    "ptvision",
                    vec![
                        "torch",
                        "torchvision",
                        "pandas",
                        "numpy",
                        "opencv",
                        "pillow",
                    ],
                ),
                (
                    "nlp",
                    vec!["torch", "nltk", "gensim", "pandas", "numpy", "scikit-learn"],
                ),
                ("async", vec!["asyncio", "aiortc", "requests"]),
            ]),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(venv_path: &Path) -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            venv_list: VenvList::new(
                Venv::from_venvs_dir(venv_path)
                    .expect("Could not create VenvList because of an error from Venv"),
            ),
            venv_index: 0,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => {
                if let crossterm::event::Event::Key(key_event) = event {
                    self.handle_key_event(key_event)?
                }
            }
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit(),
                AppEvent::ScrollDown => self.select_next(),
                AppEvent::ScrollUp => self.select_previuos(),
                AppEvent::SelectVenv => todo!(),
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::ScrollDown),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
    pub fn select_next(&mut self) {
        self.venv_list.state.select_next();
        self.update_venv_index();
    }
    pub fn select_previuos(&mut self) {
        self.venv_list.state.select_previous();
        self.update_venv_index();
    }
    pub fn select_first(&mut self) {
        self.venv_list.state.select_first();
        self.update_venv_index();
    }
    pub fn select_last(&mut self) {
        self.venv_list.state.select_last();
        self.update_venv_index();
    }
    pub fn update_venv_index(&mut self) {
        if let Some(i) = self.venv_list.state.selected() {
            if i >= self.venv_list.venvs.len() {
                self.select_first();
                return;
            } else if i == usize::MAX {
                self.select_last();
                return;
            }
            self.venv_index = i;
        }
    }
    pub fn get_selected_venv(&mut self) -> Venv {
        self.venv_list.venvs[self.venv_index].clone()
    }
}
