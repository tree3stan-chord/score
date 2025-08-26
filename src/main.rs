use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Text, Span, Line},
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};
use std::fs;
use std::io::{self, stdout};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration as StdDuration;
use serde::{Deserialize, Serialize};
use xml::writer::{EventWriter, XmlEvent};
use midir::{MidiOutput, MidiOutputConnection};

#[derive(Clone, Serialize, Deserialize)]
struct Staff {
    notes: Vec<Note>,
    cursor_position: usize,
    time_signature: TimeSignature,
    key_signature: KeySignature,
    scroll_offset: usize,
    max_length: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct TimeSignature {
    numerator: u8,
    denominator: u8,
}

#[derive(Clone, Serialize, Deserialize)]
struct KeySignature {
    sharps: i8, // positive for sharps, negative for flats
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
    tuplet_mode: Option<(u8, u8)>,
    history: Vec<Score>,
    history_index: usize,
    midi_out: Option<Arc<Mutex<MidiOutputConnection>>>,
    is_playing: bool,
    setup_mode: bool,
    setup_field: SetupField,
    viewport_width: usize,
    scroll_animation: f32,
}

#[derive(Clone, PartialEq)]
enum SetupField {
    TimeNumerator,
    TimeDenominator, 
    KeySignature,
    StaffCount,
    Done,
}

#[derive(Clone, Serialize, Deserialize)]
struct Note {
    pitch: Pitch,
    duration: Duration,
    accidental: Accidental,
    position: usize,
    beam_group: Option<usize>,
    tied_to_next: bool,
    tuplet_group: Option<TupletGroup>,
}

#[derive(Clone, Serialize, Deserialize)]
struct TupletGroup {
    ratio: (u8, u8), // (3, 2) for triplets, (5, 4) for quintuplets
    group_id: usize,
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
    Sixteenth,
}

impl App {
    fn new() -> Self {
        let initial_staves = vec![Staff {
            notes: Vec::new(),
            cursor_position: 0,
            time_signature: TimeSignature { numerator: 4, denominator: 4 },
            key_signature: KeySignature { sharps: 0 },
            scroll_offset: 0,
            max_length: 64, // Start with 64 beats, can expand
        }];
        
        let midi_out = Self::init_midi().ok();
        
        Self {
            staves: initial_staves.clone(),
            current_staff: 0,
            current_duration: Duration::Quarter,
            current_accidental: Accidental::Natural,
            filename: "score.json".to_string(),
            tuplet_mode: None,
            history: vec![Score { staves: initial_staves }],
            history_index: 0,
            midi_out,
            is_playing: false,
            setup_mode: true,
            setup_field: SetupField::TimeNumerator,
            viewport_width: 16, // Default viewport shows 16 beats
            scroll_animation: 0.0,
        }
    }

    fn move_cursor_right(&mut self) {
        let current_staff_idx = self.current_staff;
        if let Some(staff) = self.staves.get_mut(current_staff_idx) {
            if staff.cursor_position < staff.max_length - 1 {
                staff.cursor_position += 1;
                
                // Auto-expand staff if approaching the end
                if staff.cursor_position > staff.max_length - 8 {
                    staff.max_length += 16; // Expand by 16 beats
                }
            }
        }
        self.update_viewport();
    }

    fn move_cursor_left(&mut self) {
        let current_staff_idx = self.current_staff;
        if let Some(staff) = self.staves.get_mut(current_staff_idx) {
            if staff.cursor_position > 0 {
                staff.cursor_position -= 1;
            }
        }
        self.update_viewport();
    }
    
    fn update_viewport(&mut self) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            let cursor_pos = staff.cursor_position;
            let current_scroll = staff.scroll_offset;
            let _viewport_end = current_scroll + self.viewport_width;
            
            // Scroll right if cursor moves beyond 75% of viewport
            if cursor_pos >= current_scroll + (self.viewport_width * 3 / 4) {
                staff.scroll_offset = cursor_pos.saturating_sub(self.viewport_width / 4);
                self.scroll_animation = 1.0;
            }
            // Scroll left if cursor moves before 25% of viewport
            else if cursor_pos < current_scroll + (self.viewport_width / 4) {
                staff.scroll_offset = cursor_pos.saturating_sub(self.viewport_width * 3 / 4);
                self.scroll_animation = -1.0;
            }
            
            // Ensure we don't scroll past the beginning
            staff.scroll_offset = staff.scroll_offset.min(staff.max_length.saturating_sub(self.viewport_width));
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
        self.save_to_history();
        
        let current_staff_idx = self.current_staff;
        if let Some(staff) = self.staves.get_mut(current_staff_idx) {
            let cursor_pos = staff.cursor_position;
            staff.notes.retain(|note| note.position != cursor_pos);
            
            let beam_group = if matches!(self.current_duration, Duration::Eighth | Duration::Sixteenth) {
                Self::calculate_beam_group_static(staff, cursor_pos)
            } else {
                None
            };
            
            let tuplet_group = if let Some(tuplet_ratio) = self.tuplet_mode {
                Some(TupletGroup {
                    ratio: tuplet_ratio,
                    group_id: cursor_pos / 3,
                })
            } else {
                None
            };
            
            staff.notes.push(Note {
                pitch,
                duration: self.current_duration.clone(),
                accidental: self.current_accidental.clone(),
                position: cursor_pos,
                beam_group,
                tied_to_next: false,
                tuplet_group,
            });
        }
    }

    fn cycle_duration(&mut self) {
        self.current_duration = match self.current_duration {
            Duration::Whole => Duration::Half,
            Duration::Half => Duration::Quarter,
            Duration::Quarter => Duration::Eighth,
            Duration::Eighth => Duration::Sixteenth,
            Duration::Sixteenth => Duration::Whole,
        };
    }

    fn delete_note(&mut self) {
        self.save_to_history();
        
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
                time_signature: TimeSignature { numerator: 4, denominator: 4 },
                key_signature: KeySignature { sharps: 0 },
                scroll_offset: 0,
                max_length: 64,
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

    fn calculate_beam_group_static(staff: &Staff, position: usize) -> Option<usize> {
        let adjacent_eighth_notes: Vec<&Note> = staff.notes.iter()
            .filter(|note| {
                matches!(note.duration, Duration::Eighth | Duration::Sixteenth) &&
                note.position.abs_diff(position) <= 2
            })
            .collect();
        
        if adjacent_eighth_notes.len() > 0 {
            Some(position / 4)
        } else {
            None
        }
    }

    fn toggle_tie(&mut self) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            if let Some(note) = staff.notes.iter_mut().find(|n| n.position == staff.cursor_position) {
                note.tied_to_next = !note.tied_to_next;
            }
        }
    }
    
    fn toggle_triplet_mode(&mut self) {
        self.tuplet_mode = match self.tuplet_mode {
            None => Some((3, 2)),
            Some((3, 2)) => Some((5, 4)),
            Some((5, 4)) => None,
            _ => None,
        };
    }
    
    fn cycle_time_signature(&mut self) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            staff.time_signature = match (staff.time_signature.numerator, staff.time_signature.denominator) {
                (4, 4) => TimeSignature { numerator: 3, denominator: 4 },
                (3, 4) => TimeSignature { numerator: 2, denominator: 4 },
                (2, 4) => TimeSignature { numerator: 6, denominator: 8 },
                (6, 8) => TimeSignature { numerator: 4, denominator: 4 },
                _ => TimeSignature { numerator: 4, denominator: 4 },
            };
        }
    }
    
    fn cycle_key_signature(&mut self) {
        if let Some(staff) = self.staves.get_mut(self.current_staff) {
            staff.key_signature.sharps = match staff.key_signature.sharps {
                s if s < 7 => s + 1,
                7 => -7,
                s if s < 0 => s + 1,
                _ => 0,
            };
        }
    }
    
    fn save_to_history(&mut self) {
        let current_score = Score { staves: self.staves.clone() };
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(current_score);
        self.history_index = self.history.len() - 1;
        
        // Keep history reasonable size
        if self.history.len() > 50 {
            self.history.remove(0);
            self.history_index = self.history.len() - 1;
        }
    }
    
    fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.staves = self.history[self.history_index].staves.clone();
            if self.current_staff >= self.staves.len() {
                self.current_staff = self.staves.len().saturating_sub(1);
            }
        }
    }
    
    fn redo(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.staves = self.history[self.history_index].staves.clone();
            if self.current_staff >= self.staves.len() {
                self.current_staff = self.staves.len().saturating_sub(1);
            }
        }
    }
    
    fn export_musicxml(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut output = Vec::new();
        {
            let mut writer = EventWriter::new(&mut output);
            
            writer.write(XmlEvent::start_element("score-partwise").attr("version", "3.1"))?;
            
            // Part list
            writer.write(XmlEvent::start_element("part-list"))?;
            for (i, _) in self.staves.iter().enumerate() {
                let part_id = format!("P{}", i + 1);
                writer.write(XmlEvent::start_element("score-part").attr("id", &part_id))?;
                writer.write(XmlEvent::start_element("part-name"))?;
                writer.write(XmlEvent::characters(&format!("Staff {}", i + 1)))?;
                writer.write(XmlEvent::end_element())?; // part-name
                writer.write(XmlEvent::end_element())?; // score-part
            }
            writer.write(XmlEvent::end_element())?; // part-list
            
            // Parts
            for (staff_idx, staff) in self.staves.iter().enumerate() {
                let part_id = format!("P{}", staff_idx + 1);
                writer.write(XmlEvent::start_element("part").attr("id", &part_id))?;
                
                // Measure
                writer.write(XmlEvent::start_element("measure").attr("number", "1"))?;
                
                // Attributes (time signature, key signature)
                writer.write(XmlEvent::start_element("attributes"))?;
                
                writer.write(XmlEvent::start_element("time"))?;
                writer.write(XmlEvent::start_element("beats"))?;
                writer.write(XmlEvent::characters(&staff.time_signature.numerator.to_string()))?;
                writer.write(XmlEvent::end_element())?;
                writer.write(XmlEvent::start_element("beat-type"))?;
                writer.write(XmlEvent::characters(&staff.time_signature.denominator.to_string()))?;
                writer.write(XmlEvent::end_element())?;
                writer.write(XmlEvent::end_element())?; // time
                
                if staff.key_signature.sharps != 0 {
                    writer.write(XmlEvent::start_element("key"))?;
                    writer.write(XmlEvent::start_element("fifths"))?;
                    writer.write(XmlEvent::characters(&staff.key_signature.sharps.to_string()))?;
                    writer.write(XmlEvent::end_element())?;
                    writer.write(XmlEvent::end_element())?; // key
                }
                
                writer.write(XmlEvent::end_element())?; // attributes
                
                // Notes
                for note in &staff.notes {
                    writer.write(XmlEvent::start_element("note"))?;
                    
                    writer.write(XmlEvent::start_element("pitch"))?;
                    let (step, octave) = match note.pitch {
                        Pitch::C4 => ("C", "4"),
                        Pitch::D4 => ("D", "4"),
                        Pitch::E4 => ("E", "4"),
                        Pitch::F4 => ("F", "4"),
                        Pitch::G4 => ("G", "4"),
                        Pitch::A4 => ("A", "4"),
                        Pitch::B4 => ("B", "4"),
                    };
                    writer.write(XmlEvent::start_element("step"))?;
                    writer.write(XmlEvent::characters(step))?;
                    writer.write(XmlEvent::end_element())?;
                    
                    if note.accidental != Accidental::Natural {
                        writer.write(XmlEvent::start_element("alter"))?;
                        let alter = match note.accidental {
                            Accidental::Sharp => "1",
                            Accidental::Flat => "-1",
                            _ => "0",
                        };
                        writer.write(XmlEvent::characters(alter))?;
                        writer.write(XmlEvent::end_element())?;
                    }
                    
                    writer.write(XmlEvent::start_element("octave"))?;
                    writer.write(XmlEvent::characters(octave))?;
                    writer.write(XmlEvent::end_element())?;
                    writer.write(XmlEvent::end_element())?; // pitch
                    
                    writer.write(XmlEvent::start_element("duration"))?;
                    let duration_value = match note.duration {
                        Duration::Whole => "4",
                        Duration::Half => "2",
                        Duration::Quarter => "1",
                        Duration::Eighth => "0.5",
                        Duration::Sixteenth => "0.25",
                    };
                    writer.write(XmlEvent::characters(duration_value))?;
                    writer.write(XmlEvent::end_element())?;
                    
                    writer.write(XmlEvent::start_element("type"))?;
                    let type_name = match note.duration {
                        Duration::Whole => "whole",
                        Duration::Half => "half",
                        Duration::Quarter => "quarter",
                        Duration::Eighth => "eighth",
                        Duration::Sixteenth => "16th",
                    };
                    writer.write(XmlEvent::characters(type_name))?;
                    writer.write(XmlEvent::end_element())?;
                    
                    writer.write(XmlEvent::end_element())?; // note
                }
                
                writer.write(XmlEvent::end_element())?; // measure
                writer.write(XmlEvent::end_element())?; // part
            }
            
            writer.write(XmlEvent::end_element())?; // score-partwise
        }
        
        let xml_string = String::from_utf8(output)?;
        fs::write("score.musicxml", xml_string)?;
        Ok(())
    }
    
    fn init_midi() -> Result<Arc<Mutex<MidiOutputConnection>>, Box<dyn std::error::Error>> {
        let midi_out = MidiOutput::new("SCORE MIDI Output")?;
        let out_ports = midi_out.ports();
        
        let port = out_ports.get(0).ok_or("No MIDI output port available")?;
        let conn_out = midi_out.connect(port, "score-playback")?;
        
        Ok(Arc::new(Mutex::new(conn_out)))
    }
    
    fn play_note(&self, note: &Note) {
        if let Some(midi_out) = &self.midi_out {
            if let Ok(mut conn) = midi_out.lock() {
                let midi_note = match note.pitch {
                    Pitch::C4 => 60,
                    Pitch::D4 => 62,
                    Pitch::E4 => 64,
                    Pitch::F4 => 65,
                    Pitch::G4 => 67,
                    Pitch::A4 => 69,
                    Pitch::B4 => 71,
                };
                
                let adjusted_note = match note.accidental {
                    Accidental::Sharp => midi_note + 1,
                    Accidental::Flat => midi_note - 1,
                    Accidental::Natural => midi_note,
                };
                
                // Note on
                let _ = conn.send(&[0x90, adjusted_note, 64]);
                
                // Schedule note off
                let midi_out_clone = Arc::clone(&midi_out);
                let note_duration = match note.duration {
                    Duration::Whole => 2000,
                    Duration::Half => 1000,
                    Duration::Quarter => 500,
                    Duration::Eighth => 250,
                    Duration::Sixteenth => 125,
                };
                
                thread::spawn(move || {
                    thread::sleep(StdDuration::from_millis(note_duration));
                    if let Ok(mut conn) = midi_out_clone.lock() {
                        let _ = conn.send(&[0x80, adjusted_note, 64]);
                    }
                });
            }
        }
    }
    
    fn toggle_playback(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.start_playback();
        }
    }
    
    fn start_playback(&mut self) {
        if let Some(staff) = self.staves.get(self.current_staff) {
            let mut notes_to_play: Vec<Note> = staff.notes.clone();
            notes_to_play.sort_by_key(|n| n.position);
            
            for note in notes_to_play {
                if !self.is_playing { break; }
                self.play_note(&note);
                thread::sleep(StdDuration::from_millis(100)); // Small gap between notes
            }
        }
        self.is_playing = false;
    }
    
    fn handle_setup_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                self.setup_field = match self.setup_field {
                    SetupField::TimeNumerator => SetupField::TimeDenominator,
                    SetupField::TimeDenominator => SetupField::KeySignature,
                    SetupField::KeySignature => SetupField::StaffCount,
                    SetupField::StaffCount => SetupField::Done,
                    SetupField::Done => {
                        self.setup_mode = false;
                        self.setup_field = SetupField::Done;
                        return;
                    }
                };
            }
            KeyCode::Up => {
                if let Some(staff) = self.staves.get_mut(0) {
                    match self.setup_field {
                        SetupField::TimeNumerator => {
                            staff.time_signature.numerator = (staff.time_signature.numerator % 12) + 1;
                        }
                        SetupField::TimeDenominator => {
                            staff.time_signature.denominator = match staff.time_signature.denominator {
                                2 => 4, 4 => 8, 8 => 16, _ => 2,
                            };
                        }
                        SetupField::KeySignature => {
                            staff.key_signature.sharps = (staff.key_signature.sharps + 1).clamp(-7, 7);
                        }
                        SetupField::StaffCount => {
                            if self.staves.len() < 4 {
                                self.add_staff();
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Down => {
                if let Some(staff) = self.staves.get_mut(0) {
                    match self.setup_field {
                        SetupField::TimeNumerator => {
                            staff.time_signature.numerator = if staff.time_signature.numerator > 1 {
                                staff.time_signature.numerator - 1
                            } else {
                                12
                            };
                        }
                        SetupField::TimeDenominator => {
                            staff.time_signature.denominator = match staff.time_signature.denominator {
                                16 => 8, 8 => 4, 4 => 2, _ => 16,
                            };
                        }
                        SetupField::KeySignature => {
                            staff.key_signature.sharps = (staff.key_signature.sharps - 1).clamp(-7, 7);
                        }
                        SetupField::StaffCount => {
                            if self.staves.len() > 1 {
                                self.remove_staff();
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Esc => {
                self.setup_mode = false;
            }
            _ => {}
        }
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
        
        // Decay scroll animation
        if app.scroll_animation != 0.0 {
            app.scroll_animation *= 0.8; // Fade animation
            if app.scroll_animation.abs() < 0.1 {
                app.scroll_animation = 0.0;
            }
        }

        if let Event::Key(key) = event::read()? {
            if app.setup_mode {
                app.handle_setup_input(key.code);
                continue;
            }
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
                KeyCode::Char('t') => app.toggle_tie(),
                KeyCode::Char('3') => app.toggle_triplet_mode(),
                KeyCode::Char('T') => app.cycle_time_signature(),
                KeyCode::Char('K') => app.cycle_key_signature(),
                KeyCode::Char('z') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.undo(),
                KeyCode::Char('y') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.redo(),
                KeyCode::Char('x') => { let _ = app.export_musicxml(); },
                KeyCode::Char('p') => app.toggle_playback(),
                KeyCode::F(1) => { app.setup_mode = true; app.setup_field = SetupField::TimeNumerator; },
                KeyCode::Home => {
                    if let Some(staff) = app.staves.get_mut(app.current_staff) {
                        staff.cursor_position = 0;
                        staff.scroll_offset = 0;
                        app.scroll_animation = -1.0;
                    }
                },
                KeyCode::End => {
                    if let Some(staff) = app.staves.get_mut(app.current_staff) {
                        staff.cursor_position = staff.max_length.saturating_sub(1);
                        app.update_viewport();
                        app.scroll_animation = 1.0;
                    }
                },
                KeyCode::PageUp => {
                    if let Some(staff) = app.staves.get_mut(app.current_staff) {
                        let jump = app.viewport_width / 2;
                        staff.cursor_position = staff.cursor_position.saturating_sub(jump);
                        app.update_viewport();
                        app.scroll_animation = -1.0;
                    }
                },
                KeyCode::PageDown => {
                    if let Some(staff) = app.staves.get_mut(app.current_staff) {
                        let jump = app.viewport_width / 2;
                        staff.cursor_position = (staff.cursor_position + jump).min(staff.max_length - 1);
                        app.update_viewport();
                        app.scroll_animation = 1.0;
                    }
                },
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    if app.setup_mode {
        render_setup_screen(f, app);
        return;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.area());

    let staff_content = render_all_staves(app);
    let current_staff = &app.staves[app.current_staff];
    let scroll_info = if current_staff.max_length > app.viewport_width {
        format!(" | Pos: {}-{}/{}", 
                current_staff.scroll_offset + 1, 
                (current_staff.scroll_offset + app.viewport_width).min(current_staff.max_length),
                current_staff.max_length)
    } else {
        String::new()
    };
    
    let staff_block = Block::default()
        .title(format!(
            "🎼 Score - Staff {}/{} | {}/{} Time | {} Key{}{}",
            app.current_staff + 1,
            app.staves.len(),
            current_staff.time_signature.numerator,
            current_staff.time_signature.denominator,
            format_key_signature(&current_staff.key_signature),
            scroll_info,
            if app.is_playing { " ♪ PLAYING ♪" } else { "" }
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    let staff = Paragraph::new(staff_content)
        .block(staff_block);
    
    f.render_widget(staff, chunks[0]);

    let tuplet_text = match app.tuplet_mode {
        Some((3, 2)) => " | 3=triplets",
        Some((5, 4)) => " | 3=quintuplets", 
        _ => " | 3=tuplets(off)",
    };
    
    let help_text = format!(
        "Controls: ←→=cursor, ↑↓=staff, Home/End=jump, PgUp/PgDn=scroll, notes=c/d/e/f/g/a/b, SPACE=duration({}), #=accidental({}), t=tie, F1=setup{}, +=add, -=remove, p=play, x=export, s=save, l=load, DEL=delete, q=quit",
        match app.current_duration {
            Duration::Whole => "whole",
            Duration::Half => "half", 
            Duration::Quarter => "quarter",
            Duration::Eighth => "eighth",
            Duration::Sixteenth => "sixteenth",
        },
        match app.current_accidental {
            Accidental::Natural => "natural",
            Accidental::Sharp => "sharp",
            Accidental::Flat => "flat",
        },
        tuplet_text
    );
    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL));
    
    f.render_widget(help, chunks[1]);
}

fn render_setup_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(10), // Setup form
            Constraint::Min(2),     // Instructions
        ])
        .split(f.area());
    
    // Title
    let title = Paragraph::new(
        Line::from(vec![
            Span::styled("🎼 SCORE Setup - Configure Your Score", 
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ])
    )
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
    .style(Style::default().bg(Color::Black));
    
    f.render_widget(title, chunks[0]);
    
    // Setup form
    let staff = app.staves.get(0).unwrap();
    let current_field = &app.setup_field;
    
    let form_lines = vec![
        Line::from(vec![
            Span::styled("Time Signature: ", Style::default().fg(Color::White)),
            Span::styled(
                format!(" {}/{}  ", staff.time_signature.numerator, staff.time_signature.denominator),
                if *current_field == SetupField::TimeNumerator || *current_field == SetupField::TimeDenominator {
                    Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                }
            )
        ]),
        Line::from(" "),
        Line::from(vec![
            Span::styled("Key Signature:  ", Style::default().fg(Color::White)),
            Span::styled(
                format!(" {}  ", format_key_signature(&staff.key_signature)),
                if *current_field == SetupField::KeySignature {
                    Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                }
            )
        ]),
        Line::from(" "),
        Line::from(vec![
            Span::styled("Staff Count:    ", Style::default().fg(Color::White)),
            Span::styled(
                format!(" {}  ", app.staves.len()),
                if *current_field == SetupField::StaffCount {
                    Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                }
            )
        ]),
        Line::from(" "),
        Line::from(vec![
            Span::styled(
                if *current_field == SetupField::Done { "▶ START COMPOSING" } else { "  Start Composing  " },
                if *current_field == SetupField::Done {
                    Style::default().bg(Color::Green).fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                }
            )
        ]),
    ];
    
    let form = Paragraph::new(form_lines)
        .block(Block::default().borders(Borders::ALL).title("Settings").border_style(Style::default().fg(Color::Green)));
    
    f.render_widget(form, chunks[1]);
    
    // Instructions
    let instructions = vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("↑/↓ = Adjust values  |  "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" = Next field  |  "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" = Skip setup")
        ]),
        Line::from(" "),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Cyan)),
            Span::raw("You can also press F1 anytime during composition to return to this setup screen.")
        ]),
    ];
    
    let help = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Help").border_style(Style::default().fg(Color::Cyan)));
    
    f.render_widget(help, chunks[2]);
}

fn render_all_staves(app: &App) -> Text<'_> {
    let mut all_lines = Vec::new();
    
    for (staff_idx, staff) in app.staves.iter().enumerate() {
        if staff_idx > 0 {
            all_lines.push("".to_string());
        }
        
        let staff_lines = render_single_staff(staff, staff_idx == app.current_staff, app.viewport_width);
        all_lines.extend(staff_lines);
    }
    
    // Add animation hint if scrolling
    if app.scroll_animation != 0.0 {
        let scroll_hint = if app.scroll_animation > 0.0 {
            "                    >>> Scrolling Right >>>"
        } else {
            "                    <<< Scrolling Left <<<"
        };
        all_lines.push("".to_string());
        all_lines.push(scroll_hint.to_string());
    }
    
    Text::raw(all_lines.join("\n"))
}

fn render_single_staff(staff: &Staff, is_current: bool, viewport_width: usize) -> Vec<String> {
    let line_char = if is_current { "─" } else { "·" };
    let space_char = if is_current { " " } else { " " };
    
    // Calculate visible range based on scroll offset
    let start_pos = staff.scroll_offset;
    let end_pos = (start_pos + viewport_width).min(staff.max_length);
    let visible_width = (end_pos - start_pos) * 5;
    
    let mut staff_lines = vec![
        format!("{}", space_char.repeat(visible_width)), // E5 line
        format!("{}", space_char.repeat(visible_width)), // D5 space
        format!("{}", line_char.repeat(visible_width)), // C5 line
        format!("{}", space_char.repeat(visible_width)), // B4 space
        format!("{}", line_char.repeat(visible_width)), // A4 line
        format!("{}", space_char.repeat(visible_width)), // G4 space
        format!("{}", line_char.repeat(visible_width)), // F4 line
        format!("{}", space_char.repeat(visible_width)), // E4 space
        format!("{}", line_char.repeat(visible_width)), // D4 line
        format!("{}", space_char.repeat(visible_width)), // C4 space
        format!("{}", line_char.repeat(visible_width)), // B3 line
    ];
    
    // Add default rests to empty positions in visible range
    for position in start_pos..end_pos {
        let pos = (position - start_pos) * 5;
        let has_note = staff.notes.iter().any(|note| note.position == position);
        if !has_note && pos < staff_lines[8].len() {
            staff_lines[8].replace_range(pos..pos+1, "𝄽"); // Quarter rest on middle line
        }
    }

    // Render notes in visible range
    for note in &staff.notes {
        if note.position < start_pos || note.position >= end_pos {
            continue; // Skip notes outside viewport
        }
        
        let line_idx = match note.pitch {
            Pitch::C4 => 9,
            Pitch::D4 => 8,
            Pitch::E4 => 7,
            Pitch::F4 => 6,
            Pitch::G4 => 5,
            Pitch::A4 => 4,
            Pitch::B4 => 3,
        };
        
        let pos = (note.position - start_pos) * 5;
        if pos < staff_lines[line_idx].len() {
            let note_symbol = match note.duration {
                Duration::Whole => "○",
                Duration::Half => "♩", 
                Duration::Quarter => "●",
                Duration::Eighth => if note.beam_group.is_some() { "♪" } else { "♫" },
                Duration::Sixteenth => "♬",
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
            
            if note.tied_to_next {
                let tie_pos = pos + 1;
                if tie_pos < staff_lines[line_idx].len() {
                    staff_lines[line_idx].replace_range(tie_pos..tie_pos+1, "⌢");
                }
            }
        }
    }

    // Render cursor if it's in visible range
    if is_current && staff.cursor_position >= start_pos && staff.cursor_position < end_pos {
        let cursor_pos = (staff.cursor_position - start_pos) * 5;
        if cursor_pos < staff_lines[0].len() {
            for line in staff_lines.iter_mut() {
                if line.chars().nth(cursor_pos).unwrap_or(' ') == ' ' {
                    line.replace_range(cursor_pos..cursor_pos+1, "║");
                }
            }
        }
    }
    
    // Add scroll indicators
    if is_current {
        if start_pos > 0 {
            // Left scroll indicator
            for line in staff_lines.iter_mut() {
                if line.len() > 0 {
                    line.replace_range(0..1, "◀");
                }
            }
        }
        if end_pos < staff.max_length {
            // Right scroll indicator
            for line in staff_lines.iter_mut() {
                let last_pos = line.len().saturating_sub(1);
                if last_pos > 0 {
                    line.replace_range(last_pos..last_pos+1, "▶");
                }
            }
        }
    }

    staff_lines
}

fn format_key_signature(key_sig: &KeySignature) -> String {
    match key_sig.sharps {
        0 => "C major".to_string(),
        1 => "G major (1#)".to_string(),
        2 => "D major (2#)".to_string(),
        3 => "A major (3#)".to_string(),
        4 => "E major (4#)".to_string(),
        5 => "B major (5#)".to_string(),
        6 => "F# major (6#)".to_string(),
        7 => "C# major (7#)".to_string(),
        -1 => "F major (1♭)".to_string(),
        -2 => "B♭ major (2♭)".to_string(),
        -3 => "E♭ major (3♭)".to_string(),
        -4 => "A♭ major (4♭)".to_string(),
        -5 => "D♭ major (5♭)".to_string(),
        -6 => "G♭ major (6♭)".to_string(),
        -7 => "C♭ major (7♭)".to_string(),
        _ => "Unknown".to_string(),
    }
}