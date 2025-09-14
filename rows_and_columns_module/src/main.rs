// src/main.rs

/// Main entry point for rows_and_columns CSV analysis tool
/// 
/// This application provides terminal-based CSV data analysis and visualization
/// with directory-based data storage for scalability. Follows the "do one thing well"
/// philosophy with modular design.
///
/// # Usage
/// ```bash
/// rows_and_columns
/// ```
///
/// # Features
/// - Loads CSV files with persistent directory-based storage
/// - Provides pandas-style descriptive statistics
/// - Renders ASCII/Unicode TUI charts (histogram, scatter, box-plot)
/// - Integrates with FF file manager for file selection
/// - Memory-efficient streaming data processing (no pre-loading)

// ... doc comments ...

mod error_types_module;
mod manage_absolute_executable_directory_relative_paths;
mod rows_and_columns_module;
mod csv_processor_module;
use rows_and_columns_module::run_rows_and_columns_application;

/// Application entry point - delegates to primary module
/// 
/// # Returns
/// - `Ok(())` on successful execution
/// - `Err(String)` with error description on failure
/// 
/// # Examples
/// ```rust
/// // This is called automatically when binary is executed
/// fn main() -> Result<(), String> {
///     run_rows_and_columns_application()
/// }
/// ```
fn main() {
    // Call the primary module function with comprehensive error handling
    if let Err(error_message) = run_rows_and_columns_application() {
        
        // Display error to user with clear context
        eprintln!("rows_and_columns application error: {}", error_message);
        
        // Exit with error code indicating failure
        std::process::exit(1);
    }
    
    // Successful execution - exit with success code
    std::process::exit(0);
}
