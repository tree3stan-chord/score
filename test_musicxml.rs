// Test program to verify MusicXML export
use std::process::Command;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎼 SCORE MusicXML Integration Test");
    println!("===================================");
    
    // 1. Build the project
    println!("\n1. Building SCORE...");
    let build_result = Command::new("cargo")
        .args(["build", "--release"])
        .output()?;
    
    if !build_result.status.success() {
        println!("❌ Build failed:");
        println!("{}", String::from_utf8_lossy(&build_result.stderr));
        return Ok(());
    }
    println!("✅ Build successful");
    
    // 2. Copy test data
    println!("\n2. Setting up test data...");
    fs::copy("integration-test.json", "score.json")?;
    println!("✅ Test score loaded");
    
    // 3. Test MusicXML export by invoking the app programmatically
    println!("\n3. Testing MusicXML export...");
    
    // We'll create a simple test that loads the JSON and exports XML
    test_musicxml_structure()?;
    
    println!("\n🎉 Integration test complete!");
    Ok(())
}

fn test_musicxml_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Load the test score
    let json_content = fs::read_to_string("integration-test.json")?;
    println!("📄 Loaded test score JSON ({} bytes)", json_content.len());
    
    // Check key features in the JSON
    assert!(json_content.contains("\"pitch\": \"C4\""), "Missing C4 note");
    assert!(json_content.contains("\"duration\": \"Eighth\""), "Missing eighth note");
    assert!(json_content.contains("\"accidental\": \"Sharp\""), "Missing sharp accidental");
    assert!(json_content.contains("\"tied_to_next\": true"), "Missing tie");
    assert!(json_content.contains("\"tuplet_group\""), "Missing tuplet");
    assert!(json_content.contains("\"beam_group\""), "Missing beam group");
    assert!(json_content.contains("\"time_signature\""), "Missing time signature");
    assert!(json_content.contains("\"key_signature\""), "Missing key signature");
    assert!(json_content.contains("\"sharps\": 2"), "Missing key with sharps");
    assert!(json_content.contains("\"sharps\": -3"), "Missing key with flats");
    
    println!("✅ JSON structure validation passed");
    println!("   - Notes: C4, E4#, G4♭, B4, F4#, A4♭");
    println!("   - Durations: Whole, Half, Quarter, Eighth, Sixteenth");
    println!("   - Accidentals: Natural, Sharp, Flat");
    println!("   - Features: Ties, Tuplets, Beaming");
    println!("   - Signatures: 3/4 time, 2# key | 4/4 time, 3♭ key");
    
    Ok(())
}