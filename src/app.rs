use std::{path::PathBuf, process::Command};

use crate::{
    event::{AppEvent, Event, EventHandler},
    venv::{Venv, VenvList, model::Package},
};
use crossterm::event::KeyEventKind;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,
    /// List of virtual environments
    pub venv_list: VenvList,
    pub venv_index: usize,
    pub packages_index: usize,
    pub current_focus: Panel,
    output: Output,
}

#[derive(Debug)]
pub enum Panel {
    Venv,
    Packages,
}

#[derive(Debug)]
pub enum Output {
    /// path of the selected venv
    VenvPath(PathBuf),
    /// requirement of the selected venv
    Requirements(String),
    /// nothing
    None,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(venvs: Vec<Venv>) -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            // TODO: constructor should receive the venv_list.
            // it does not care about how venv_list is created
            venv_list: VenvList::new(venvs),
            venv_index: 0,
            current_focus: Panel::Venv,
            packages_index: 0,
            output: Output::None,
        }
    }

    // Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<Output> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(self.output)
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<Output> {
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
                AppEvent::SwitchLeft => self.switch_left(),
                AppEvent::SwitchRight => self.switch_right(),
                AppEvent::SelectVenv => {
                    let v = self.get_selected_venv_ref();
                    let venv_path = v.activation_path();
                    self.output = Output::VenvPath(venv_path);
                    self.quit();
                }
                AppEvent::Requirements => {
                    let v = self.get_selected_venv_ref();
                    // TODO: terrible error handling here. fix it. probs show the error message in
                    // the TUI
                    // TODO: confirmation as well
                    let python = v.requirements();
                    let output = Command::new(python)
                        .args(["-m", "pip", "freeze"])
                        .output()?;

                    let req = String::from_utf8(output.stdout)
                        .expect("Could not create string from output.stdout");

                    self.output = Output::Requirements(req);
                    self.quit();
                }
            },
        }
        Ok(Output::None)
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.kind != KeyEventKind::Press {
            // return early for Release and Repeat
            // Windows handles release as well
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::ScrollDown),
            KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::SwitchRight),
            KeyCode::Left | KeyCode::Char('h') => self.events.send(AppEvent::SwitchLeft),
            KeyCode::Char('a') => self.events.send(AppEvent::SelectVenv),
            KeyCode::Char('r') => self.events.send(AppEvent::Requirements),
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
    // === Switching Panels ===
    pub fn switch_left(&mut self) {
        self.current_focus = Panel::Venv;
    }

    pub fn switch_right(&mut self) {
        self.current_focus = Panel::Packages;
    }

    // === List Operations ===
    pub fn select_next(&mut self) {
        match self.current_focus {
            Panel::Venv => {
                self.venv_list.list_state.select_next();
                self.update_venv_index();
            }
            Panel::Packages => {
                let current_venv = self.get_selected_venv_ref();
                current_venv.list_state.select_next();
                self.update_package_index();
            }
        }
    }
    pub fn select_previuos(&mut self) {
        match self.current_focus {
            Panel::Venv => {
                self.venv_list.list_state.select_previous();
                self.update_venv_index();
            }
            Panel::Packages => {
                let current_venv = self.get_selected_venv_ref();
                current_venv.list_state.select_previous();
                self.update_package_index();
            }
        }
    }
    pub fn select_first(&mut self) {
        match self.current_focus {
            Panel::Venv => {
                self.venv_list.list_state.select_first();
                self.update_venv_index();
            }
            Panel::Packages => {
                let current_venv = self.get_selected_venv_ref();
                current_venv.list_state.select_first();
                self.update_package_index();
            }
        }
    }
    pub fn select_last(&mut self) {
        match self.current_focus {
            Panel::Venv => {
                self.venv_list.list_state.select_last();
                self.update_venv_index();
            }
            Panel::Packages => {
                let current_venv = self.get_selected_venv_ref();
                current_venv.list_state.select_last();
                self.update_package_index();
            }
        }
    }
    pub fn update_venv_index(&mut self) {
        if let Some(i) = self.venv_list.list_state.selected() {
            if i >= self.venv_list.venvs.len() {
                self.select_first();
                return;
            } else if i == usize::MAX {
                self.select_last();
                return;
            }
            self.venv_index = i;
            // reset the package index when venv changes
            self.packages_index = 0;
        }
    }
    pub fn update_package_index(&mut self) {
        let current_venv = self.get_selected_venv_ref();
        if let Some(i) = current_venv.list_state.selected() {
            if i == usize::MAX {
                self.select_last();
                return;
            } else if i >= current_venv.packages.len() {
                self.select_first();
                return;
            }
            self.packages_index = i;
        }
    }
    pub fn get_selected_venv(&mut self) -> Venv {
        self.venv_list.venvs[self.venv_index].clone()
    }
    pub fn get_selected_venv_ref(&mut self) -> &mut Venv {
        &mut self.venv_list.venvs[self.venv_index]
    }
    pub fn get_selected_package(&mut self) -> Package {
        let v = self.get_selected_venv();
        v.packages[self.packages_index].clone()
    }
}
