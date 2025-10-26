// src/ui/display.rs - Display utilities

use colored::*;

pub fn print_banner() {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║           NEXUS GITHUB ORCHESTRATOR v2.0                  ║".cyan().bold());
    println!("{}", "║          Multi-Account GitHub Actions Runner              ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
    println!();
}

pub fn print_section(title: &str) {
    println!("\n{}", format!("═══ {} ═══", title).yellow().bold());
}

pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message.red());
}

pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️ ".yellow(), message.yellow());
}

pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️ ".cyan(), message);
}

pub fn print_menu_item(number: usize, label: &str) {
    println!("  {} {}", format!("[{}]", number).cyan().bold(), label);
}

pub fn print_submenu_item(number: usize, label: &str) {
    println!("    {} {}", format!("{}", number).white(), label);
}

pub fn print_separator() {
    println!("{}", "─────────────────────────────────────────────────────────".bright_black());
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn pause() {
    use std::io::{self, Write};
    print!("\n{}", "Press Enter to continue...".bright_black());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
