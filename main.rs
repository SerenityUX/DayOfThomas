use serde_json::Value;
use std::fs;
use colored::*;
use chrono::{NaiveDate, Datelike, Local};
use std::collections::HashMap;
use std::io::{self, Write};

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

fn main() {
    // Read the JSON file
    let data = fs::read_to_string("analysis.json")
        .expect("Unable to read file");
    
    // Parse the JSON string
    let json: Value = serde_json::from_str(&data)
        .expect("Unable to parse JSON");
    
    if let Value::Array(entries) = json {
        let entry_count = entries.len();
        
        // Create a HashMap of dates to colors
        let mut date_colors = HashMap::new();
        for entry in &entries {
            if let (Some(date), color) = (
                entry["dateCreated"].as_str(),
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
            entry["dateCreated"].as_str() == Some(&today)
        });

        if has_today_entry {
            println!("You have an entry for today's journal");
        } else {
            print!("Press enter to create a journal entry");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            // Here you can add the logic to create a new journal entry
            println!("\nCreating new journal entry...");
        }
    }
}
