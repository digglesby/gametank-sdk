use std::time::Duration;

use rat_widget::menu::{popup_menu, PopupMenu, PopupMenuState};
use ratatui::{crossterm::event::{Event, KeyCode, KeyEvent}, layout::Alignment, style::{Color, Modifier, Style, Stylize}, symbols::border::{self}, widgets::{block::Position, Block, List, ListDirection, ListState, Widget}, Frame};

use crate::{helpers::{centered_rect, SCHEME}, ui::quickmenu::QuickMenu, Component};

struct Item { label: &'static str, enabled: bool }

pub struct Menu {
    has_podman: bool,
    quit: bool,
    qm: QuickMenu,
}

impl Menu {
    pub fn init() -> Self {
        let has_podman = false;
        let qm = QuickMenu::init(vec!["_Emulator", "_Tracker", "ROM _Flasher", "_Build"]);

        Self {
            has_podman,
            quit: false,
            qm,
        }
    }
}


impl Component for Menu {
    fn render(&mut self, frame: &mut Frame) {
        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .title("─ GameTank GO! ")
            .title_style(SCHEME.style(Color::Rgb(36, 36, 36)).italic().bold());
        block.render(frame.area(), frame.buffer_mut());
        self.qm.render(frame);
    }

    fn should_exit(&self) -> bool {
        self.qm.should_exit()
    }
    
    fn update(&mut self, events: Vec<Event>) {
        self.qm.update(events);
    }
}
