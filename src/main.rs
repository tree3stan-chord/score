use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};
use std::io::{self, stdout};

struct App {
    cursor_position: usize,
    notes: Vec<Note>,
    current_duration: Duration,
    current_accidental: Accidental,
}

#[derive(Clone)]
struct Note {
    pitch: Pitch,
    duration: Duration,
    accidental: Accidental,
    position: usize,
}

#[derive(Clone)]
enum Pitch {
    C4, D4, E4, F4, G4, A4, B4,
}

#[derive(Clone, PartialEq)]
enum Accidental {
    Natural,
    Sharp,
    Flat,
}

#[derive(Clone, PartialEq)]
enum Duration {
    Whole,
    Half,
    Quarter,
    Eighth,
}

impl App {
    fn new() -> Self {
        Self {
            cursor_position: 0,
            notes: Vec::new(),
            current_duration: Duration::Quarter,
            current_accidental: Accidental::Natural,
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < 15 {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn add_note(&mut self, pitch: Pitch) {
        self.notes.retain(|note| note.position != self.cursor_position);
        self.notes.push(Note {
            pitch,
            duration: self.current_duration.clone(),
            accidental: self.current_accidental.clone(),
            position: self.cursor_position,
        });
    }

    fn cycle_duration(&mut self) {
        self.current_duration = match self.current_duration {
            Duration::Whole => Duration::Half,
            Duration::Half => Duration::Quarter,
            Duration::Quarter => Duration::Eighth,
            Duration::Eighth => Duration::Whole,
        };
    }

    fn delete_note(&mut self) {
        self.notes.retain(|note| note.position != self.cursor_position);
    }

    fn cycle_accidental(&mut self) {
        self.current_accidental = match self.current_accidental {
            Accidental::Natural => Accidental::Sharp,
            Accidental::Sharp => Accidental::Flat,
            Accidental::Flat => Accidental::Natural,
        };
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => app.move_cursor_left(),
                KeyCode::Right => app.move_cursor_right(),
                KeyCode::Char('c') => app.add_note(Pitch::C4),
                KeyCode::Char('d') => app.add_note(Pitch::D4),
                KeyCode::Char('e') => app.add_note(Pitch::E4),
                KeyCode::Char('f') => app.add_note(Pitch::F4),
                KeyCode::Char('g') => app.add_note(Pitch::G4),
                KeyCode::Char('a') => app.add_note(Pitch::A4),
                KeyCode::Char('b') => app.add_note(Pitch::B4),
                KeyCode::Char(' ') => app.cycle_duration(),
                KeyCode::Delete | KeyCode::Backspace => app.delete_note(),
                KeyCode::Char('#') => app.cycle_accidental(),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.area());

    let staff_block = Block::default()
        .title("Score")
        .borders(Borders::ALL);
    
    let staff_content = render_staff(app);
    let staff = Paragraph::new(staff_content)
        .block(staff_block);
    
    f.render_widget(staff, chunks[0]);

    let help_text = format!(
        "Controls: Arrow keys=move, c/d/e/f/g/a/b=notes, SPACE=duration({}), #=accidental({}), DEL=delete, q=quit",
        match app.current_duration {
            Duration::Whole => "whole",
            Duration::Half => "half", 
            Duration::Quarter => "quarter",
            Duration::Eighth => "eighth",
        },
        match app.current_accidental {
            Accidental::Natural => "natural",
            Accidental::Sharp => "sharp",
            Accidental::Flat => "flat",
        }
    );
    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL));
    
    f.render_widget(help, chunks[1]);
}

fn render_staff(app: &App) -> Text<'_> {
    let mut staff_lines = vec![
        "     ".repeat(16), // E5 line
        "     ".repeat(16), // D5 space
        "─────".repeat(16), // C5 line
        "     ".repeat(16), // B4 space
        "─────".repeat(16), // A4 line
        "     ".repeat(16), // G4 space
        "─────".repeat(16), // F4 line
        "     ".repeat(16), // E4 space
        "─────".repeat(16), // D4 line
        "     ".repeat(16), // C4 space
        "─────".repeat(16), // B3 line
    ];

    for note in &app.notes {
        let line_idx = match note.pitch {
            Pitch::C4 => 9,
            Pitch::D4 => 8,
            Pitch::E4 => 7,
            Pitch::F4 => 6,
            Pitch::G4 => 5,
            Pitch::A4 => 4,
            Pitch::B4 => 3,
        };
        
        let pos = note.position * 5;
        if pos < staff_lines[line_idx].len() {
            let note_symbol = match note.duration {
                Duration::Whole => "○",
                Duration::Half => "♩", 
                Duration::Quarter => "●",
                Duration::Eighth => "♫",
            };
            
            let accidental_symbol = match note.accidental {
                Accidental::Natural => "",
                Accidental::Sharp => "#",
                Accidental::Flat => "♭",
            };
            
            if !accidental_symbol.is_empty() && pos > 0 {
                staff_lines[line_idx].replace_range(pos-1..pos, accidental_symbol);
            }
            staff_lines[line_idx].replace_range(pos..pos+1, note_symbol);
        }
    }

    let cursor_pos = app.cursor_position * 5;
    if cursor_pos < staff_lines[0].len() {
        for line in staff_lines.iter_mut() {
            if line.chars().nth(cursor_pos) == Some(' ') {
                line.replace_range(cursor_pos..cursor_pos+1, "│");
            }
        }
    }

    Text::raw(staff_lines.join("\n"))
}