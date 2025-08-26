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
use std::fs;
use std::io::{self, stdout};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Staff {
    notes: Vec<Note>,
    cursor_position: usize,
}

#[derive(Serialize, Deserialize)]
struct Score {
    staves: Vec<Staff>,
}

struct App {
    staves: Vec<Staff>,
    current_staff: usize,
    current_duration: Duration,
    current_accidental: Accidental,
    filename: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Note {
    pitch: Pitch,
    duration: Duration,
    accidental: Accidental,
    position: usize,
}

#[derive(Clone, Serialize, Deserialize)]
enum Pitch {
    C4, D4, E4, F4, G4, A4, B4,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Accidental {
    Natural,
    Sharp,
    Flat,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Duration {
    Whole,
    Half,
    Quarter,
    Eighth,
}

impl App {
    fn new() -> Self {
        Self {
            staves: vec![Staff {
                notes: Vec::new(),
                cursor_position: 0,
            }],
            current_staff: 0,
            current_duration: Duration::Quarter,
            current_accidental: Accidental::Natural,
            filename: "score.json".to_string(),
        }
    }

    fn move_cursor_right(&mut self) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            if staff.cursor_position < 15 {
                staff.cursor_position += 1;
            }
        }
    }

    fn move_cursor_left(&mut self) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            if staff.cursor_position > 0 {
                staff.cursor_position -= 1;
            }
        }
    }

    fn move_staff_up(&mut self) {
        if self.current_staff > 0 {
            self.current_staff -= 1;
        }
    }

    fn move_staff_down(&mut self) {
        if self.current_staff + 1 < self.staves.len() {
            self.current_staff += 1;
        }
    }

    fn add_note(&mut self, pitch: Pitch) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            staff.notes.retain(|note| note.position != staff.cursor_position);
            staff.notes.push(Note {
                pitch,
                duration: self.current_duration.clone(),
                accidental: self.current_accidental.clone(),
                position: staff.cursor_position,
            });
        }
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
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            staff.notes.retain(|note| note.position != staff.cursor_position);
        }
    }

    fn cycle_accidental(&mut self) {
        self.current_accidental = match self.current_accidental {
            Accidental::Natural => Accidental::Sharp,
            Accidental::Sharp => Accidental::Flat,
            Accidental::Flat => Accidental::Natural,
        };
    }

    fn add_staff(&mut self) {
        if self.staves.len() < 4 {
            self.staves.push(Staff {
                notes: Vec::new(),
                cursor_position: 0,
            });
        }
    }

    fn remove_staff(&mut self) {
        if self.staves.len() > 1 {
            self.staves.remove(self.current_staff);
            if self.current_staff >= self.staves.len() {
                self.current_staff = self.staves.len() - 1;
            }
        }
    }

    fn save_score(&self) -> Result<(), Box<dyn std::error::Error>> {
        let score = Score {
            staves: self.staves.clone(),
        };
        let json = serde_json::to_string_pretty(&score)?;
        fs::write(&self.filename, json)?;
        Ok(())
    }

    fn load_score(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(contents) = fs::read_to_string(&self.filename) {
            let score: Score = serde_json::from_str(&contents)?;
            self.staves = score.staves;
            if self.current_staff >= self.staves.len() {
                self.current_staff = 0;
            }
        }
        Ok(())
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
                KeyCode::Up => app.move_staff_up(),
                KeyCode::Down => app.move_staff_down(),
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
                KeyCode::Char('s') => { let _ = app.save_score(); },
                KeyCode::Char('l') => { let _ = app.load_score(); },
                KeyCode::Char('+') => app.add_staff(),
                KeyCode::Char('-') => app.remove_staff(),
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

    let staff_content = render_all_staves(app);
    let staff_block = Block::default()
        .title(format!("Score - Staff {}/{}", app.current_staff + 1, app.staves.len()))
        .borders(Borders::ALL);
    let staff = Paragraph::new(staff_content)
        .block(staff_block);
    
    f.render_widget(staff, chunks[0]);

    let help_text = format!(
        "Controls: ←→=move cursor, ↑↓=change staff, c/d/e/f/g/a/b=notes, SPACE=duration({}), #=accidental({}), +=add staff, -=remove staff, s=save, l=load, DEL=delete, q=quit",
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

fn render_all_staves(app: &App) -> Text<'_> {
    let mut all_lines = Vec::new();
    
    for (staff_idx, staff) in app.staves.iter().enumerate() {
        if staff_idx > 0 {
            all_lines.push("".to_string());
        }
        
        let staff_lines = render_single_staff(staff, staff_idx == app.current_staff);
        all_lines.extend(staff_lines);
    }
    
    Text::raw(all_lines.join("\n"))
}

fn render_single_staff(staff: &Staff, is_current: bool) -> Vec<String> {
    let line_char = if is_current { "─" } else { "·" };
    let space_char = if is_current { " " } else { " " };
    
    let mut staff_lines = vec![
        format!("{}", space_char.repeat(80)), // E5 line
        format!("{}", space_char.repeat(80)), // D5 space
        format!("{}", line_char.repeat(16).repeat(5)), // C5 line
        format!("{}", space_char.repeat(80)), // B4 space
        format!("{}", line_char.repeat(16).repeat(5)), // A4 line
        format!("{}", space_char.repeat(80)), // G4 space
        format!("{}", line_char.repeat(16).repeat(5)), // F4 line
        format!("{}", space_char.repeat(80)), // E4 space
        format!("{}", line_char.repeat(16).repeat(5)), // D4 line
        format!("{}", space_char.repeat(80)), // C4 space
        format!("{}", line_char.repeat(16).repeat(5)), // B3 line
    ];

    for note in &staff.notes {
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

    if is_current {
        let cursor_pos = staff.cursor_position * 5;
        if cursor_pos < staff_lines[0].len() {
            for line in staff_lines.iter_mut() {
                if line.chars().nth(cursor_pos).unwrap_or(' ') == ' ' {
                    line.replace_range(cursor_pos..cursor_pos+1, "│");
                }
            }
        }
    }

    staff_lines
}