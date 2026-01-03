//! Windows uninstaller executable
//!
//! This is a standalone executable that removes Pulsar from Windows.
//! It reads uninstall metadata and calls the WindowsInstaller uninstall method.

use std::path::PathBuf;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    println!("Pulsar Uninstaller");
    println!("==================\n");

    // Get the installation directory (this executable should be in it)
    let exe_path = std::env::current_exe().expect("Failed to get executable path");
    let install_dir = exe_path.parent().expect("Failed to get install directory");

    println!("Uninstalling from: {}\n", install_dir.display());

    // Read uninstall metadata
    let metadata_path = install_dir.join("uninstall_metadata.json");
    if !metadata_path.exists() {
        eprintln!("Error: Uninstall metadata not found at: {}", metadata_path.display());
        eprintln!("Cannot proceed with uninstallation.");
        std::process::exit(1);
    }

    // Confirm with user
    println!("This will remove Pulsar and all its components.");
    println!("Are you sure you want to continue? (y/N): ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");
    
    if input.trim().to_lowercase() != "y" {
        println!("\nUninstallation cancelled.");
        std::process::exit(0);
    }

    println!("\nStarting uninstallation...\n");

    // Run the async uninstall
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let result = runtime.block_on(async {
        use pulsar_installer::{Uninstaller, traits::{Progress, ProgressCallback}};

        let uninstaller = Uninstaller::from_metadata(&metadata_path)
            .expect("Failed to load uninstall metadata");

        let progress: ProgressCallback = Box::new(|p: Progress| {
            println!("[{:3.0}%] {}", p.current, p.message.unwrap_or("Working..."));
        });

        uninstaller.uninstall(progress).await
    });

    match result {
        Ok(_) => {
            println!("\n✓ Uninstallation complete!");
            println!("\nYou can now close this window.");
            
            // Wait for user to press Enter before closing
            println!("\nPress Enter to exit...");
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
        }
        Err(e) => {
            eprintln!("\n✗ Uninstallation failed: {}", e);
            eprintln!("\nPress Enter to exit...");
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            std::process::exit(1);
        }
    }
}
