use serde_json::{Value, json};
use std::fs;
use colored::*;
use chrono::{NaiveDate, Datelike, Local};
use std::collections::HashMap;
use std::io::{self, Write};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use std::thread;
use reqwest::multipart;
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

fn get_openai_key() -> String {
    dotenv().ok();
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in .env file")
}

async fn transcribe_audio(audio_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let file_bytes = tokio::fs::read(audio_path).await?;
    let file_part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(audio_path.to_string())
        .mime_str("audio/wav")?;

    let form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", "whisper-1")
        .text("language", "en")
        .text("response_format", "text");

    let response = client.post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", get_openai_key()))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("API request failed: {}", error_text).into());
    }

    let transcript = response.text().await?;
    Ok(transcript)
}

// GitHub's contribution colors (from light to dark)
const COLORS: [(u8, u8, u8); 5] = [
    (235, 237, 240), // Empty
    (155, 233, 168), // Light
    (64, 196, 99),   // Medium
    (48, 161, 78),   // Dark
    (33, 110, 57)    // Very Dark
];

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
    (r, g, b)
}

fn record_audio(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create audio directory if it doesn't exist
    let mut audio_dir = get_exe_dir();
    audio_dir.push("audio");
    fs::create_dir_all(&audio_dir)?;
    
    let host = cpal::default_host();
    let device = host.default_input_device()
        .expect("No input device available");

    let mut config = device.default_input_config()?.config();
    config.channels = 1;

    println!("\nRecording... Press Enter to stop.");

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let recording = Arc::new(AtomicBool::new(true));
    let recording_clone = recording.clone();

    let sample_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let sample_count_clone = sample_count.clone();

    let writer = Arc::new(std::sync::Mutex::new(Some(
        hound::WavWriter::create(filename, spec)?
    )));

    let writer_clone = writer.clone();

    let err_fn = move |err| {
        eprintln!("An error occurred on stream: {}", err);
    };

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &_| {
            if let Some(writer) = &mut *writer_clone.lock().unwrap() {
                for &sample in data {
                    // Convert f32 to i16
                    let sample = (sample * i16::MAX as f32) as i16;
                    writer.write_sample(sample).unwrap();
                    sample_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        },
        err_fn,
        None
    )?;

    stream.play()?;

    // Wait for Enter key in a separate thread
    thread::spawn(move || {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        recording_clone.store(false, Ordering::SeqCst);
    });

    // Print time while recording
    while recording.load(Ordering::SeqCst) {
        let current_samples = sample_count.load(Ordering::SeqCst);
        print!("\rRecording: {:.1} seconds", 
               current_samples as f32 / config.sample_rate.0 as f32);
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(100));
    }

    drop(stream);
    
    // Take ownership of the writer and finalize it
    if let Some(writer) = writer.lock().unwrap().take() {
        writer.finalize()?;
    }
    
    println!("\nRecording saved.");
    Ok(())
}

async fn get_color_from_gpt(transcript: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", get_openai_key()))
        .json(&json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a color expert. Based on the emotional content and mood of the text provided, return only a hex color code that best represents it. Return only the hex code, nothing else."
                },
                {
                    "role": "user",
                    "content": transcript
                }
            ],
            "max_tokens": 10
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("GPT API request failed: {}", error_text).into());
    }

    let response_json: Value = response.json().await?;
    let color = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("#9BE9A8")
        .trim()
        .to_string();

    Ok(color)
}

fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .expect("Failed to get executable path")
        .parent()
        .expect("Failed to get executable directory")
        .to_path_buf()
}

fn get_audio_path(date: &str) -> PathBuf {
    let mut path = get_exe_dir();
    path.push("audio");
    path.push(format!("{}.wav", date));
    path
}

fn get_analysis_path() -> PathBuf {
    let mut path = get_exe_dir();
    path.push("analysis.json");
    path
}

async fn create_journal_entry(date: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Record audio
    let audio_path = get_audio_path(date);
    record_audio(audio_path.to_str().unwrap())?;

    // Get transcript
    let transcript = match transcribe_audio(audio_path.to_str().unwrap()).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Warning: Transcription failed: {}", e);
            String::new()
        }
    };

    // Get color from GPT
    let color = match get_color_from_gpt(&transcript).await {
        Ok(color) => color,
        Err(e) => {
            eprintln!("Warning: Color analysis failed: {}", e);
            "#FFFFFF".to_string()
        }
    };

    // Read existing entries
    let analysis_path = get_analysis_path();
    let file_content = std::fs::read_to_string(&analysis_path)?;
    let mut entries: Vec<serde_json::Value> = serde_json::from_str(&file_content)?;

    // Create new entry
    let entry = serde_json::json!({
        "date": date,
        "colorAssociatedWithDay": color,
        "transcript": transcript,
        "audioPath": format!("audio/{}.wav", date)
    });

    // Add new entry
    entries.push(entry);

    // Write back to file
    std::fs::write(
        analysis_path,
        serde_json::to_string_pretty(&entries)?
    )?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the analysis file path
    let analysis_path = get_analysis_path();
    
    // Create parent directories if they don't exist
    if let Some(parent) = analysis_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Create analysis.json with empty array if it doesn't exist
    if !analysis_path.exists() {
        fs::write(&analysis_path, "[]")?;
    }
    
    // Read the JSON file
    let data = fs::read_to_string(&analysis_path)?;
    
    // Parse the JSON string
    let json: Value = serde_json::from_str(&data)?;
    
    if let Value::Array(entries) = json {
        let entry_count = entries.len();
        
        // Create a HashMap of dates to colors
        let mut date_colors = HashMap::new();
        for entry in &entries {
            if let (Some(date), color) = (
                entry["date"].as_str(),
                entry["colorAssociatedWithDay"].as_str()
            ) {
                date_colors.insert(date.to_string(), color.map(|s| s.to_string()));
            }
        }

        println!("\n{} contributions in the last year\n", entry_count);
        
        // Store all weeks for aligned printing
        let mut all_weeks: Vec<Vec<(Option<NaiveDate>, Option<String>)>> = Vec::new();
        let mut current_week = Vec::new();
        
        // Start from January 1st, 2025
        let mut current_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        
        // Fill in days before the first day of the year
        let initial_weekday = current_date.weekday().num_days_from_monday() as usize;
        for _ in 0..initial_weekday {
            current_week.push((None, None));
        }
        
        // Process all dates
        while current_date <= end_date {
            let date_str = current_date.format("%Y-%m-%d").to_string();
            let color = date_colors.get(&date_str).cloned().flatten();
            current_week.push((Some(current_date), color));
            
            if current_date.weekday().num_days_from_monday() == 6 {
                all_weeks.push(current_week);
                current_week = Vec::new();
            }
            
            current_date = current_date.succ_opt().unwrap();
        }
        
        // Add any remaining days
        if !current_week.is_empty() {
            while current_week.len() < 7 {
                current_week.push((None, None));
            }
            all_weeks.push(current_week);
        }
        
        // Print the contribution graph
        for weekday in 0..7 {
            for week in &all_weeks {
                let (date_opt, color_opt) = &week[weekday];
                let block = match (date_opt, color_opt) {
                    (Some(_), Some(color)) => {
                        let (r, g, b) = hex_to_rgb(color);
                        format!("{} ", "■".truecolor(r, g, b))
                    },
                    _ => format!("{} ", "□".truecolor(COLORS[0].0, COLORS[0].1, COLORS[0].2))
                };
                print!("{}", block);
            }
            println!();
        }
        
        // Print legend
        println!("");
        
        // Check for today's entry
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let has_today_entry = entries.iter().any(|entry| {
            entry["date"].as_str() == Some(&today)
        });

        if has_today_entry {
            println!("You have an entry for today's journal");
        } else {
            print!("Press enter to create a journal entry");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            println!("\nCreating new journal entry...");
            if let Err(e) = create_journal_entry(&today).await {
                eprintln!("Error creating journal entry: {}", e);
            }
        }
    }
    
    Ok(())
}
