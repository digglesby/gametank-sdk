use crossbeam_channel::{Receiver, Sender};
use rat_widget::{list::selection::RowSelection, table::{selection::CellSelection, textdata::{Cell, Row}, Table, TableData, TableDataIter, TableState}};
use ratatui::{crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers}, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Widget}};

use crate::{helpers::SCHEME, main_menu::MainMenu, Component, GlobalEvent};

pub struct Handler {
    pub event: Event,
    pub action: Box<dyn Fn()>
}

// tracker subcomponent
pub trait TSub: Component {
    fn global_handlers(&self) -> Vec<Handler>;
}

#[derive(Clone, Copy)]
pub enum TrackerCmd {
    Quit,
    Left,
    Right,
    Up,
    Down,
}

type Pattern = [[Beat; 64]; 9];

fn empty_pattern() -> Pattern {
    std::array::from_fn(|_| std::array::from_fn(|_| Beat::default()))
}

#[derive(Default, Clone)]
pub struct Beat {
    cmd_list: Vec<ChannelCmd>,
    sqc_list: Vec<SequencerCmd>
}


#[derive(Clone)]
pub enum SequencerCmd {
    Tempo(u8), // 0 - 256 in bpm. 60hz * 60s = 3600 / tempo = tick counter.
    Load(u8, u16), // load a wavetable from a pointer?
    Pattern(u8), // change to pattern #
    Beat(u8), // set next beat to beat #
    Advance, // continues to the next pattern in the sequence
    Stop, // stops the sequencer
}


#[derive(Clone)]
pub enum ChannelCmd {
    Tremolo(u8, u8), // volume
    Vibrato(u8, u8), // pitch
    Wavetable(u16), // set wavetable
    Phase(u16), // set phase
    Note(u8), // set note (freq)
    Volume(u8), // volume index (0..=16)
    SlideVol(u8, i16), // how many beats, delta
    StopVSlide,
    SlidePitch(u8, i16), // how many beats, delta
    StopPSlide,
}



pub struct TrackerData {
    beat: u8,
    pattern: u8,
    sequence: u8,

    sequences: [u8; 256],
    patterns: Vec<Pattern>,
}

pub struct Tracker {
    scroll: usize,
    row: usize,
    column: usize,

    tx_main: Sender<GlobalEvent>,
    tr_tx: Sender<TrackerCmd>,
    tr_rx: Receiver<TrackerCmd>,
    subcomponents: Vec<Box<dyn TSub>>,
    handlers: Vec<Handler>,
    data: TrackerData,
}

pub fn tx_handler(tx: &Sender<TrackerCmd>, code: KeyCode, cmd: TrackerCmd) -> Handler {
    let txx = tx.clone();
    let cmd = cmd.clone();
    Handler { event: Event::Key(KeyEvent::new(code, KeyModifiers::NONE)), action: Box::new(move || {
        let _ = txx.send(cmd);
    })}
}

impl Tracker {
    pub fn init(tx_main: Sender<GlobalEvent>) -> Self {
        let (tr_tx, tr_rx) = crossbeam_channel::unbounded();

        let mut subcomponents = Vec::new();

        // let tx1 = tr_tx.clone();
        // let tx2 = tr_tx.clone();
        let handlers = vec![
            tx_handler(&tr_tx, KeyCode::Char('q'), TrackerCmd::Quit),
            tx_handler(&tr_tx, KeyCode::Esc, TrackerCmd::Quit),
            tx_handler(&tr_tx, KeyCode::Up, TrackerCmd::Up),
            tx_handler(&tr_tx, KeyCode::Down, TrackerCmd::Down),
            tx_handler(&tr_tx, KeyCode::Left, TrackerCmd::Left),
            tx_handler(&tr_tx, KeyCode::Right, TrackerCmd::Right),
        ];

        Tracker {
            tx_main,
            tr_tx,
            tr_rx,
            subcomponents,
            handlers,
            data: TrackerData {
                beat: 0,
                pattern: 0,
                sequence: 0,
                sequences: [0; 256],
                patterns: vec![empty_pattern()],
            },
            scroll: 0,
            row: 1,
            column: 1,
        }
    }
}

impl Component for Tracker {
    fn update(&mut self, events: Vec<ratatui::crossterm::event::Event>) {
        for e in events {
            // TODO: combine iterators
            for h in &self.handlers {
                if h.event == e {
                    (h.action)()
                }
            }

            for h in self.subcomponents.iter().map(|c| c.global_handlers()).flatten() {
                if h.event == e {
                    (h.action)()
                }
            }
        }
        
        for cmd in self.tr_rx.try_iter() {
            match cmd {
                TrackerCmd::Quit => {
                    let menu = MainMenu::init(self.tx_main.clone());
                    let _ = self.tx_main.send(GlobalEvent::ChangeInterface(Box::new(menu)));
                },
                TrackerCmd::Left => {
                    if self.column > 0 {
                        self.column -= 1;
                    }
                },
                TrackerCmd::Right => {
                    if self.column < 1+3*8 {
                        self.column += 1;
                    }
                },
                TrackerCmd::Up => {
                    if self.row > 0 {
                        self.row -= 1;
                    }
                },
                TrackerCmd::Down => {
                    if self.row < 64 {
                        self.row += 1;
                    }
                },
            }
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(8),
                Constraint::Percentage(100),
            ])
            .split(frame.area());

        let blk = Block::new()
            .bg(SCHEME.true_dark_color(SCHEME.black[0]));
        
        let block1 = Block::new()
            .bg(SCHEME.true_dark_color(SCHEME.black[3]))
            .borders(Borders::TOP)
            .title(" Gametank GO! | ☆•° . * . ﾟTRACKER  ﾟ. * . °•☆ ")
            .title_alignment(Alignment::Center)
            .italic()
            .fg(SCHEME.orange[3]);

        let table = Table::default()
            .data(self)
            // .block(block2)
            .style(SCHEME
                .true_dark_black(0)
                .fg(SCHEME.white[0])
            )     
            .widths([
                Constraint::Length(7), // "  XX  " for Beat in pattern (up to 3F)
                Constraint::Length(3), // "[n] ")
                
                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects
                
                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects
                
                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects
                
                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects

                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects

                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects

                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects

                Constraint::Length(5), // 3 for note name "G#7"
                Constraint::Length(3), // 1 for volume (0..=F + M/-)
                Constraint::Length(4), // 3 "[n]" for effects
            ]);

        let mut ts = TableState::<RowSelection>::default();

        let lower_layouts = Layout::default().constraints([
            Constraint::Fill(1),
            Constraint::Length(10+((5+3+4)*8)),
            Constraint::Fill(1),
        ]).direction(Direction::Horizontal).split(layout[1]);
 
        frame.render_widget(block1.clone(), layout[0]);
        frame.render_widget(blk.clone(), layout[1]);
        frame.render_stateful_widget(table, lower_layouts[1], &mut ts);
    }
}


enum ColType {
    Beat,
    Seq,
    ChBeat(usize, usize), // channel, note/vol/fx
    Empty,
}

fn channel_header<'a>(channel: u8, color: Color) -> Vec<Cell<'a>> {
    vec![
        Cell::new(Span::from(format!("  ch{}", channel))).fg(color).italic(),
        Cell::new(Span::from(" v ")).fg(color),
        Cell::new(Span::from(":↗↘  ")).fg(color),
    ]
}

impl <'a> TableData<'a> for &mut Tracker {
    fn rows(&self) -> usize {
        64
    }
    
    fn row_height(&self, row: usize) -> u16 {
        1
    }

    fn row_style(&self, row: usize) -> Option<Style> {
        // to some calculations ...
        None
    }

    fn header(&self) -> Option<rat_widget::table::textdata::Row<'a>> {
        let c = [
            SCHEME.orange[3],
            SCHEME.red[3],
            SCHEME.magenta[3],
            SCHEME.purple[3],
            SCHEME.blue[3],
            SCHEME.cyan[3],
            SCHEME.green[3],
            SCHEME.yellow[3],
        ];

        let mut cells = vec![
            Cell::new(Span::from(" BEAT  ")),
            Cell::new(Span::from("SEQ")),
        ];

        for i in 0..8 {
            cells.append(&mut channel_header(i, c[i as usize]));
        }

        Some(Row::new(cells))
    }


    fn render_cell(
        &self,
        ctx: &rat_widget::table::TableContext,
        column: usize,
        row: usize,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {

        let col = match column {
            0 => ColType::Beat,
            1 => ColType::Seq,
            n @ 2..5 => ColType::ChBeat(0, (n-2)%3),
            n @ 5..8 => ColType::ChBeat(1, (n-2)%3),
            n @ 8..11 => ColType::ChBeat(2, (n-2)%3),
            n @ 11..14 => ColType::ChBeat(3, (n-2)%3),
            n @ 14..17 => ColType::ChBeat(4, (n-2)%3),
            n @ 17..20 => ColType::ChBeat(5, (n-2)%3),
            n @ 20..23 => ColType::ChBeat(6, (n-2)%3),
            n @ 23..26 => ColType::ChBeat(7, (n-2)%3),
            _ => ColType::Empty,
        };

        let pattern = &self.data.patterns[self.data.pattern as usize];

        let (mut before, mut cell, mut after) = match col {
            ColType::Beat => {
                // TODO: track beat scroll position
                (
                    Span::from(""),
                    Span::from(format!("   {:02X}", row)).italic().fg(SCHEME.deepblue[2]),
                    Span::from("  ")
                )
            }
            ColType::Seq => {
                (
                    Span::from(""),
                    Span::from("---").fg(SCHEME.reduced_text_color(SCHEME.white[1])),
                    Span::from("")
                )
            },
            ColType::ChBeat(ch, variant) => {
                let cmd_list = &pattern[ch+1][row].cmd_list;

                match variant {
                    0 => {
                        let result = cmd_list.iter().find(|cmd| match cmd {
                            ChannelCmd::Note(n) => true,
                            _ => false,
                        });

                        (
                            Span::from("  "),
                            match result {
                                Some(ChannelCmd::Note(n)) => Span::from("G#7").fg(SCHEME.orange[1]),
                                _ => Span::from("---").fg(SCHEME.gray[1]),
                            },
                            Span::from(""),
                        )
                    }, // note
                    1 => {
                        let result = cmd_list.iter().find(|cmd| match cmd {
                            ChannelCmd::Volume(v) => true,
                            _ => false,
                        });
                        
                        (
                            Span::from(" "),
                            match result {
                                Some(ChannelCmd::Volume(v)) => Span::from(format!("{:x}", v)).fg(SCHEME.deepblue[3]),
                                _ => Span::from("-").fg(SCHEME.gray[0]),
                            },
                            Span::from(" "),
                        )
                    }, // volume
                    2 => {
                        let n = cmd_list.iter().filter(|cmd| 
                            match cmd {
                                ChannelCmd::Tremolo(_, _) => true,
                                ChannelCmd::Vibrato(_, _) => true,
                                ChannelCmd::Wavetable(_) => true,
                                ChannelCmd::Phase(_) => true,
                                ChannelCmd::Note(_) => false,
                                ChannelCmd::Volume(_) => false,
                                ChannelCmd::SlideVol(_, _) => true,
                                ChannelCmd::StopVSlide => true,
                                ChannelCmd::SlidePitch(_, _) => true,
                                ChannelCmd::StopPSlide => true,
                            }
                        ).count();
                        
                        (
                            Span::from(""),
                            if n > 0 {
                                Span::from(format!("[{}]", n))
                            } else {
                                Span::from("---").fg(SCHEME.black[0])
                            },
                            Span::from(" "),
                        )
                    }, // fx
                    _ => { (Span::from(""),Span::from(""),Span::from("")) }
                }
            },
            ColType::Empty => (Span::from(""),Span::from(""),Span::from("")),
        };

        if row % 2 == 0 {
            let c = SCHEME.true_dark_color(SCHEME.black[3]);
            before = before.bg(c);
            cell = cell.bg(c);
            after = after.bg(c);
        } else {
            let c = SCHEME.true_dark_color(SCHEME.black[0]);
            before = before.bg(c);
            cell = cell.bg(c);
            after = after.bg(c)
        }

        if self.row == row {
            if self.column == 0 {
                let c = SCHEME.true_dark_color(SCHEME.blue[3]);
                before = before.bg(c);
                cell = cell.bg(c);
                after = after.bg(c)
            } else {
                let c = SCHEME.true_dark_color(SCHEME.blue[0]);
                before = before.bg(c);
                cell = cell.bg(c);
                after = after.bg(c)
            }
        }

        if self.column == column && self.row == row {
            // if row modifiable?
            cell = cell.add_modifier(Modifier::SLOW_BLINK).fg(SCHEME.deepblue[1]).reversed();
        }

        let line = Line::from(vec![before, cell, after]);
        line.render(area, buf);
    }
}

