use crossterm::event::{self as ctevent, Event as CTEvent, KeyCode, ModifierKeyCode, KeyEvent, KeyEventKind};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui;
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Row, Table, TableState};

mod event;
use event::{Event, EventHandler};

#[tokio::main]
async fn main() {
    enable_raw_mode();
    execute!(std::io::stderr(), EnterAlternateScreen);
    let events = EventHandler::new(100);
    let mut terminal = ratatui::Terminal::new(Backend::new(std::io::stderr())).unwrap();
    App::new(events).run(&mut terminal).await;
    execute!(std::io::stderr(), LeaveAlternateScreen);
    disable_raw_mode();
}

#[derive(Default)]
enum Mode {
    #[default]
    Normal,
    Input,
}


pub struct App {
    quit: bool,
    buffer: Vec<u8>,
    mode: Mode,
    events: EventHandler,
    control: bool,
}

impl App {
    pub fn new(events: EventHandler) -> Self {
        Self {
            quit: false,
            buffer: Vec::new(),
            mode: Mode::Normal,
            events,
            control: false,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<Backend<std::io::Stderr>>,
    ) -> std::io::Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.render(frame))?;

            self.handle_events().await;
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        use ratatui::layout::{
            Constraint::{Fill, Length, Min},
            Layout,
        };
        use ratatui::text::Line;
        use ratatui::widgets::{Block, Paragraph};

        let main_area= frame.area();

        let main_block = Block::bordered().title(Line::from(match self.mode {
            Mode::Input => "Minimal Notepad",
            Mode::Normal => "<Esc> quit - </> Edit",
        }).bold().centered());

        let buffer = String::from_utf8(self.buffer.clone())
            .expect("Invalid Char");

        let text = Text::from(format!("{}_", buffer)).white();

        frame.render_widget(Paragraph::new(text).block(main_block), main_area);
    }

    pub async fn handle_events(&mut self) -> std::io::Result<()> {
        match self.events.next().await.unwrap() {
            Event::Key(key) => { 
                match key.kind { 
                    KeyEventKind::Press => match self.mode {
                        Mode::Normal => self.normal_mode_event(key.code).await,
                        Mode::Input => self.input_mode_event(key.code).await,
                        _ => self.default_mode_event(key.code).await,
                    }
                    _=>{},
                }
            },
            _ => {}
        }
        Ok(())
    }

    pub async fn normal_mode_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.stop().await,
            KeyCode::Char('/') => self.mode = Mode::Input,
            _ => {}
        }
    }

    pub async fn stop(&mut self) {
        let _ = self.events.stop().await;
        self.quit = true;
    }

    pub async fn default_mode_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }
    }
    pub async fn input_mode_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => self.buffer.push(c as u8),
            KeyCode::Backspace => { 
                let char = self.buffer.pop();
                if let Some(mut c) = char && self.control {
                    while c == b' ' && self.buffer.len() > 0 {
                        c = self.buffer.pop().unwrap()
                    }
                }
            },
            KeyCode::Enter => self.buffer.push(b'\n'),
            KeyCode::Tab => self.buffer.extend(b"    "),
            _ => {}
        }
    }
}
