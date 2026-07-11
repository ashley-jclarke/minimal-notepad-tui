use crossterm::event::{self as ctevent, Event as CTEvent, KeyCode, ModifierKeyCode, KeyEvent, KeyEventKind};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui;
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Row, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::style::{Color, Stylize};
mod event;
use event::{Event, EventHandler};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::symbols::scrollbar::Set;



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

#[derive(Default)]
pub struct Page {
    pub mouse_pos: usize,
    pub buffer: Vec<u8>,
    pub scrollbar: ScrollbarState,
}

impl Page {
    pub fn new() -> Self {
        Self {
            scrollbar: ScrollbarState::new(100),
            buffer: Vec::new(),
            mouse_pos: 0
        }
    }
}

pub struct App {
    quit: bool,
    pages: Vec<Page>,
    page: usize,
    mode: Mode,
    events: EventHandler,
    control: bool,
}

impl App {
    pub fn new(events: EventHandler) -> Self {
        Self {
            quit: false,
            pages: vec![Page::new()],
            page: 0,
            mode: Mode::Input,
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
        
        let layout = Layout::horizontal([
            Min(0),
            Length(1),
        ]);


        let [main_area, scroll_area] = frame.area().layout(&layout);

        let main_block = Block::bordered().title(Line::from(match self.mode {
            Mode::Input => " Minimal Notepad ",
            Mode::Normal => " Minimal Notepad </> to edit ",
        }).bold().centered()).title_bottom(Line::from(format!(" Page - {} ", self.page + 1)).centered());

        let mut cbuf = self.pages[self.page].buffer.clone();
        cbuf.insert(self.pages[self.page].mouse_pos, b' ');
        cbuf[self.pages[self.page].mouse_pos] = b'_';
        let buffer = String::from_utf8(cbuf)
            .expect("Invalid Char");


        let text = Text::from(format!("{}", buffer)).white();

        frame.render_widget(Paragraph::new(text)
            .block(main_block)
            .wrap(Wrap {trim: false})
            .scroll((self.pages[self.page].scrollbar.get_position() as u16, 0)),
        main_area);


        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .track_style(Color::Yellow)
            .begin_style(Color::Green)
            .end_style(Color::Red);



        frame.render_stateful_widget(
            scrollbar,
            main_area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.pages[self.page].scrollbar,
        );
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
            KeyCode::Left => { if self.page != 0 {self.page -= 1}}
            KeyCode::Right => { self.page += 1; if self.page + 1 > self.pages.len() {self.pages.push(Page::default())}}
            KeyCode::Up => self.pages[self.page].scrollbar.prev(),
            KeyCode::Down => self.pages[self.page].scrollbar.next(),
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
        let mouse_pos = self.pages[self.page].mouse_pos;
        match key {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => {
                self.pages[self.page].buffer.insert(mouse_pos, c as u8); 
                self.pages[self.page].mouse_pos += 1;
            },
            KeyCode::Backspace => { 
                if self.pages[self.page].buffer.len() != 0 {
                    let _char = self.pages[self.page].buffer.remove(mouse_pos - 1); 
                }
                if mouse_pos != 0 {self.pages[self.page].mouse_pos -= 1};
            },
            KeyCode::Enter => {self.pages[self.page].buffer.insert(mouse_pos, b'\n');
                self.pages[self.page].mouse_pos += 1;
            },
            //KeyCode::Tab => {self.buffer.extend(b"    ")},
            KeyCode::Left => if mouse_pos != 0 {self.pages[self.page].mouse_pos -= 1;},
            KeyCode::Right => if mouse_pos < self.pages[self.page].buffer.len() {self.pages[self.page].mouse_pos += 1},
            KeyCode::Up => self.pages[self.page].scrollbar.prev(),
            KeyCode::Down => self.pages[self.page].scrollbar.next(),
            _ => {}
        }
    }
}
