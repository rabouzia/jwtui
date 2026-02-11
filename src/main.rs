use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Jwt,
    Header,
    Payload,
    SigningKey,
}

enum InputMode {
    Normal,
    Editing,
}

struct App {
    jwt: String,
    header: String,
    payload: String,
    signing_key: String,

    jwt_cursor: usize,
    header_cursor: usize,
    payload_cursor: usize,
    signing_cursor: usize,

    focus: Focus,
    input_mode: InputMode,
}

impl App {
    fn new() -> Self {
        Self {
            jwt: String::new(),
            header: String::new(),
            payload: String::new(),
            signing_key: String::new(),

            jwt_cursor: 0,
            header_cursor: 0,
            payload_cursor: 0,
            signing_cursor: 0,

            focus: Focus::Jwt,
            input_mode: InputMode::Editing,
        }
    }

    fn active_buffer(&mut self) -> &mut String {
        match self.focus {
            Focus::Jwt => &mut self.jwt,
            Focus::Header => &mut self.header,
            Focus::Payload => &mut self.payload,
            Focus::SigningKey => &mut self.signing_key,
        }
    }

    fn active_cursor(&mut self) -> &mut usize {
        match self.focus {
            Focus::Jwt => &mut self.jwt_cursor,
            Focus::Header => &mut self.header_cursor,
            Focus::Payload => &mut self.payload_cursor,
            Focus::SigningKey => &mut self.signing_cursor,
        }
    }

    fn insert_char(&mut self, c: char) {
        let cursor = *self.active_cursor();

        let buf = self.active_buffer();
        buf.insert(cursor, c);

        *self.active_cursor() += 1;
    }
    /*
    fn insert_char(&mut self, c: char) {

        let buf = self.active_buffer();
        let cursor = self.active_cursor();
        buf.insert(*cursor, c);
        // *cursor += 1;
     }

     */

    fn backspace(&mut self) {
        let cursor = *self.active_cursor();

        if cursor > 0 {
            let buf = self.active_buffer();
            buf.remove(cursor - 1);

            *self.active_cursor() -= 1;
        }
    }

    fn move_left(&mut self) {
        let c = self.active_cursor();
        *c = c.saturating_sub(1);
    }

    fn move_right(&mut self) {
        let len = self.active_buffer().len();
        let c = self.active_cursor();
        *c = (*c + 1).min(len);
    }

    fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Jwt => Focus::Header,
            Focus::Header => Focus::Payload,
            Focus::Payload => Focus::SigningKey,
            Focus::SigningKey => Focus::Jwt,
        };
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        KeyCode::Tab => self.next_focus(),
                        KeyCode::Char(c) => self.insert_char(c),
                        KeyCode::Backspace => self.backspace(),
                        KeyCode::Left => self.move_left(),
                        KeyCode::Right => self.move_right(),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn block(title: &str, focused: bool) -> Block {
        Block::bordered().title(title).border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
        ]);

        let [help, jwt_area, middle, key_area] = vertical.areas(frame.area());

        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let [header_area, payload_area] = horizontal.areas(middle);

        let help_text = match self.input_mode {
            InputMode::Normal => Line::from("q quit | e edit"),
            InputMode::Editing => Line::from("ESC normal | TAB switch panel"),
        };

        frame.render_widget(Paragraph::new(help_text), help);

        frame.render_widget(
            Paragraph::new(self.jwt.as_str())
                .block(Self::block("JWT String", self.focus == Focus::Jwt)),
            jwt_area,
        );

        frame.render_widget(
            Paragraph::new(self.header.as_str())
                .block(Self::block("Header", self.focus == Focus::Header)),
            header_area,
        );

        frame.render_widget(
            Paragraph::new(self.payload.as_str())
                .block(Self::block("Payload", self.focus == Focus::Payload)),
            payload_area,
        );

        frame.render_widget(
            Paragraph::new(self.signing_key.as_str())
                .block(Self::block("Signing Key", self.focus == Focus::SigningKey)),
            key_area,
        );

        let (area, cursor) = match self.focus {
            Focus::Jwt => (jwt_area, self.jwt_cursor),
            Focus::Header => (header_area, self.header_cursor),
            Focus::Payload => (payload_area, self.payload_cursor),
            Focus::SigningKey => (key_area, self.signing_cursor),
        };

        if matches!(self.input_mode, InputMode::Editing) {
            frame.set_cursor_position(Position::new(area.x + cursor as u16 + 1, area.y + 1));
        }
    }
}
