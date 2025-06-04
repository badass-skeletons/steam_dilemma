use anyhow::{Context, Result};
use colored::*;
use std::process::Command;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🚀 Steam Dilemma Builder".cyan().bold());
    println!("{}", "═".repeat(50).cyan());
    
    // Check if trunk is installed
    if !check_trunk_installed() {
        eprintln!("{}", "❌ Trunk is not installed!".red().bold());
        eprintln!("   Install it with: {}", "cargo install trunk".yellow());
        std::process::exit(1);
    }

    println!("{}", "🔨 Building WASM client...".green().bold());
    
    // Build the WASM client using trunk
    let trunk_status = Command::new("trunk")
        .args(["build", "--release"])
        .status()
        .context("Failed to execute trunk command")?;

    if !trunk_status.success() {
        eprintln!("{}", "❌ Client build failed!".red().bold());
        std::process::exit(1);
    }

    println!("{}", "✅ Client build successful!".green().bold());
    
    // Run the server
    println!("{}", "🚀 Starting server...".blue().bold());
    println!("   Server will be available at: {}", "http://127.0.0.1:3000".underline().cyan());
    println!("   Health endpoint: {}", "http://127.0.0.1:3000/api/health".underline().cyan());
    println!("   Press {} to stop the server", "Ctrl+C".yellow().bold());
    println!();
    
    let mut server_cmd = TokioCommand::new("cargo")
        .args(["run", "--bin", "server"])
        .spawn()
        .context("Failed to start server")?;

    // Wait a moment and check if server started successfully
    sleep(Duration::from_secs(2)).await;
    
    if let Ok(Some(exit_status)) = server_cmd.try_wait() {
        eprintln!("{}", "❌ Server failed to start!".red().bold());
        eprintln!("   Exit status: {}", exit_status);
        std::process::exit(1);
    }

    println!("{}", "✅ Server is running!".green().bold());

    // Wait for the server to finish (it runs indefinitely)
    let _status = server_cmd.wait().await?;
    
    Ok(())
}

fn check_trunk_installed() -> bool {
    Command::new("trunk")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
} 