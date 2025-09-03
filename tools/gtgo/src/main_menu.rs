use std::time::Duration;

use crossbeam_channel::Sender;
use rat_widget::menu::{popup_menu, PopupMenu, PopupMenuState};
use ratatui::{crossterm::event::{Event, KeyCode, KeyEvent}, layout::Alignment, style::{Color, Modifier, Style, Stylize}, symbols::border::{self}, widgets::{block::Position, Block, List, ListDirection, ListState, Widget}, Frame};

use crate::{helpers::{centered_rect, SCHEME}, ui::quickmenu::{qi, QuickMenu}, Component, GlobalEvent};

pub struct MainMenu {
    has_podman: bool,
    quit: bool,
    qm: QuickMenu,
    tx: Sender<GlobalEvent>
}

impl MainMenu {
    pub fn init(tx: Sender<GlobalEvent>) -> Self {
        // TODO: if has podman
        let has_podman = false;

        let qm = QuickMenu::init(vec![
            qi("_Emulator", true, || { todo!() }),
            qi("_Tracker", true, || { todo!() }),
            qi("_Build", has_podman, || { todo!() }),
            qi("ROM _Flasher", true, || { todo!() }),
        ]);

        Self {
            has_podman,
            quit: false,
            qm,
            tx,
        }
    }
}


impl Component for MainMenu {
    fn render(&mut self, frame: &mut Frame) {
        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .title("─ GameTank GO! ")
            .title_style(SCHEME.style(Color::Rgb(36, 36, 36)).italic().bold());
        block.render(frame.area(), frame.buffer_mut());
        self.qm.render(frame);
    }
    
    fn update(&mut self, events: Vec<Event>) {
        self.qm.update(events);

        if !self.qm.is_active() {
            let _ = self.tx.send(GlobalEvent::Quit);
        }
    }
}
