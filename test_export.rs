// Test MusicXML export functionality directly
use serde::{Deserialize, Serialize};
use xml::writer::{EventWriter, XmlEvent};
use std::fs;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
enum Duration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum Pitch {
    C4, D4, E4, F4, G4, A4, B4,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
enum Accidental {
    Natural,
    Sharp,
    Flat,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct TupletGroup {
    ratio: (u8, u8),
    group_id: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Note {
    pitch: Pitch,
    duration: Duration,
    accidental: Accidental,
    position: usize,
    beam_group: Option<usize>,
    tied_to_next: bool,
    tuplet_group: Option<TupletGroup>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct TimeSignature {
    numerator: u8,
    denominator: u8,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct KeySignature {
    sharps: i8,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Staff {
    notes: Vec<Note>,
    cursor_position: usize,
    time_signature: TimeSignature,
    key_signature: KeySignature,
}

#[derive(Serialize, Deserialize, Debug)]
struct Score {
    staves: Vec<Staff>,
}

fn export_musicxml(score: &Score) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    {
        let mut writer = EventWriter::new(&mut output);
        
        writer.write(XmlEvent::start_element("score-partwise").attr("version", "3.1"))?;
        
        // Part list
        writer.write(XmlEvent::start_element("part-list"))?;
        for (i, _) in score.staves.iter().enumerate() {
            let part_id = format!("P{}", i + 1);
            writer.write(XmlEvent::start_element("score-part").attr("id", &part_id))?;
            writer.write(XmlEvent::start_element("part-name"))?;
            writer.write(XmlEvent::characters(&format!("Staff {}", i + 1)))?;
            writer.write(XmlEvent::end_element())?; // part-name
            writer.write(XmlEvent::end_element())?; // score-part
        }
        writer.write(XmlEvent::end_element())?; // part-list
        
        // Parts
        for (staff_idx, staff) in score.staves.iter().enumerate() {
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
                
                // Add accidentals if present
                if note.accidental != Accidental::Natural {
                    writer.write(XmlEvent::start_element("accidental"))?;
                    let acc_name = match note.accidental {
                        Accidental::Sharp => "sharp",
                        Accidental::Flat => "flat",
                        _ => "natural",
                    };
                    writer.write(XmlEvent::characters(acc_name))?;
                    writer.write(XmlEvent::end_element())?;
                }
                
                // Add ties if present
                if note.tied_to_next {
                    writer.write(XmlEvent::start_element("tie").attr("type", "start"))?;
                    writer.write(XmlEvent::end_element())?;
                }
                
                writer.write(XmlEvent::end_element())?; // note
            }
            
            writer.write(XmlEvent::end_element())?; // measure
            writer.write(XmlEvent::end_element())?; // part
        }
        
        writer.write(XmlEvent::end_element())?; // score-partwise
    }
    
    Ok(String::from_utf8(output)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎼 Testing MusicXML Export Functionality");
    println!("========================================");
    
    // Load test score
    let json_content = fs::read_to_string("integration-test.json")?;
    let score: Score = serde_json::from_str(&json_content)?;
    
    println!("📄 Loaded score with {} staves", score.staves.len());
    for (i, staff) in score.staves.iter().enumerate() {
        println!("   Staff {}: {} notes, {}/{} time, {} key", 
                 i + 1, 
                 staff.notes.len(),
                 staff.time_signature.numerator,
                 staff.time_signature.denominator,
                 staff.key_signature.sharps);
    }
    
    // Export to MusicXML
    println!("\n🔄 Exporting to MusicXML...");
    let xml_output = export_musicxml(&score)?;
    
    // Save to file
    fs::write("test-output.musicxml", &xml_output)?;
    println!("✅ MusicXML exported to test-output.musicxml");
    
    // Analyze the output
    println!("\n📊 MusicXML Analysis:");
    println!("   Size: {} bytes", xml_output.len());
    println!("   Contains score-partwise: {}", xml_output.contains("score-partwise"));
    println!("   Contains part-list: {}", xml_output.contains("part-list"));
    println!("   Contains time signature: {}", xml_output.contains("<beats>3</beats>"));
    println!("   Contains key signature: {}", xml_output.contains("<fifths>2</fifths>"));
    println!("   Contains notes: {}", xml_output.matches("<note>").count());
    println!("   Contains accidentals: {}", xml_output.contains("<accidental>"));
    println!("   Contains ties: {}", xml_output.contains("<tie"));
    
    // Show first few lines
    println!("\n📜 First 20 lines of MusicXML output:");
    for (i, line) in xml_output.lines().take(20).enumerate() {
        println!("   {:2}: {}", i + 1, line);
    }
    
    println!("\n🎉 MusicXML export test completed successfully!");
    Ok(())
}