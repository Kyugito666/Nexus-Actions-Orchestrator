// src/ui/input.rs - Input utilities

use std::io::{self, Write};
use anyhow::{Result, bail};

pub fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_string()
}

pub fn read_number(prompt: &str, min: usize, max: usize) -> Result<usize> {
    loop {
        let input = read_input(prompt);
        
        match input.parse::<usize>() {
            Ok(num) if num >= min && num <= max => return Ok(num),
            Ok(num) => {
                eprintln!("Number must be between {} and {}", min, max);
            }
            Err(_) => {
                eprintln!("Invalid number, please try again");
            }
        }
    }
}

pub fn read_yes_no(prompt: &str) -> bool {
    loop {
        let input = read_input(&format!("{} (y/n): ", prompt));
        
        match input.to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                eprintln!("Please enter 'y' or 'n'");
            }
        }
    }
}

pub fn read_selection(prompt: &str, options: &[&str]) -> Result<usize> {
    println!("\n{}", prompt);
    
    for (i, option) in options.iter().enumerate() {
        println!("  {}. {}", i + 1, option);
    }
    
    println!("  0. Cancel");
    println!();
    
    let choice = read_number("Select option: ", 0, options.len())?;
    
    if choice == 0 {
        bail!("Operation cancelled by user");
    }
    
    Ok(choice - 1)
}
