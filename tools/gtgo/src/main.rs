pub mod main_menu;
pub mod helpers;
pub mod ui;

use ratatui::{crossterm::event::Event, DefaultTerminal, Frame};
use anyhow::{bail, Ok, Result};

use crate::{helpers::poll_events, main_menu::Menu};

pub trait Component {
    fn update(&mut self, events: Vec<Event>);
    fn render(&mut self, frame: &mut Frame);
    fn should_exit(&self) -> bool;
}


pub struct GtGo {
    terminal: DefaultTerminal,
    state: Box<dyn Component>
}

impl GtGo {
    fn run(&mut self) -> Result<()> {
        let _ = self.terminal.draw(|f| {
            let events = poll_events();
            self.state.update(events);
            self.state.render(f); // unhandled error
        });

        if self.state.should_exit() {
            bail!("Exit")
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(terminal: DefaultTerminal) -> Result<()> {
    let mut app = GtGo { 
        terminal, 
        state: Box::new(Menu::init())
    };
    
    loop {
        app.run()?
    }
}
