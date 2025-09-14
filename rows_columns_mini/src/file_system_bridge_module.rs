
// src/file_system_bridge_module.rs

/// File system operations and CSV file selection interface for rows_and_columns
/// 
/// This module provides file browsing and selection capabilities specifically
/// tailored for CSV file import operations. It integrates with the FF-style
/// interface patterns while focusing on CSV file management.
/// 
/// # Core Responsibilities
/// - Browse file system for CSV files using FF-style interface
/// - Validate CSV file accessibility and basic format
/// - Provide file selection UI for CSV import operations
/// - Bridge between file system operations and CSV processing
/// 
/// # Design Philosophy
/// - Minimal, focused interface following FF patterns
/// - Clear file validation before processing attempts
/// - User-friendly error messages for file access issues
/// - Integration with binary-relative path management

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};

// Import our custom error types for comprehensive error handling
use super::error_types_module::{
    RowsAndColumnsError,
    RowsAndColumnsResult,
    create_file_system_error,
    create_configuration_error,
};

// Import path management for consistent file operations
use super::manage_absolute_executable_directory_relative_paths::{
    make_file_path_abs_executabledirectoryrelative_canonicalized_or_error,
    abs_executable_directory_relative_exists,
};

/// Information about a discovered CSV file
/// 
/// This structure holds all relevant information about a CSV file that
/// has been found and validated for potential import operations.
#[derive(Debug, Clone)]
pub struct CsvFileInformation {
    /// Absolute path to the CSV file
    pub absolute_file_path: PathBuf,
    
    /// Just the filename without directory path
    pub filename_only: String,
    
    /// File size in bytes
    pub file_size_bytes: u64,
    
    /// Human-readable file size (e.g., "1.2 MB", "456 KB")
    pub file_size_human_readable: String,
    
    /// Whether the file appears to be accessible for reading
    pub is_readable: bool,
}

/// Launches an interactive CSV file selection interface
/// 
/// This function provides a simple, FF-style terminal interface for browsing
/// and selecting CSV files for import. It handles user input and file validation.
/// 
/// # Returns
/// * `RowsAndColumnsResult<Option<CsvFileInformation>>` - Selected file info or None if cancelled
/// 
/// # Errors
/// * `RowsAndColumnsError::FileSystemError` - If file system access fails
/// * `RowsAndColumnsError::ConfigurationError` - If no CSV files are found
pub fn launch_csv_file_selection_interface() -> RowsAndColumnsResult<Option<CsvFileInformation>> {
    // Display the file selection header
    display_csv_file_selection_header();
    
    // Get current working directory to start browsing
    let current_directory = std::env::current_dir()
        .map_err(|io_error| {
            create_file_system_error(
                "Failed to determine current working directory for file browsing",
                io_error
            )
        })?;
    
    // Start the interactive file selection process
    interactive_directory_and_file_selection(&current_directory)
}

/// Displays the header for the CSV file selection interface
/// 
/// This provides clear instructions to the user about how to use the file
/// selection interface, following FF-style conventions.
fn display_csv_file_selection_header() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  CSV File Selection - rows_and_columns");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Select a CSV file to import and analyze:");
    println!("  • Enter number to select file/directory");
    println!("  • 'b' = back to parent directory");
    println!("  • 'q' = quit file selection");
    println!("  • Enter = refresh current directory");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
}

/// Interactive directory browsing and file selection
/// 
/// This function implements the core file browsing loop, allowing users to
/// navigate directories and select CSV files using a numbered interface.
/// 
/// # Arguments
/// * `starting_directory` - The directory to begin browsing from
/// 
/// # Returns
/// * `RowsAndColumnsResult<Option<CsvFileInformation>>` - Selected file or None if cancelled
fn interactive_directory_and_file_selection(
    starting_directory: &Path
) -> RowsAndColumnsResult<Option<CsvFileInformation>> {
    let mut current_directory = starting_directory.to_path_buf();
    
    loop {
        // Display current directory contents
        let directory_items = scan_directory_for_navigation(&current_directory)?;
        
        if directory_items.is_empty() {
            println!("No accessible files or directories found in this location.");
            println!("Press Enter to try refreshing, or 'b' to go back, 'q' to quit.");
        } else {
            display_directory_contents(&current_directory, &directory_items);
        }
        
        // Get user input
        print!("Selection: ");
        io::stdout().flush().map_err(|io_error| {
            create_file_system_error("Failed to flush stdout for user input", io_error)
        })?;
        
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).map_err(|io_error| {
            create_file_system_error("Failed to read user input", io_error)
        })?;
        
        let user_input = user_input.trim();
        
        // Process user input
        match process_user_file_selection_input(user_input, &directory_items, &mut current_directory)? {
            FileSelectionAction::ContinueBrowsing => {
                // Continue the loop
                continue;
            }
            FileSelectionAction::FileSelected(csv_file_info) => {
                return Ok(Some(csv_file_info));
            }
            FileSelectionAction::QuitSelection => {
                return Ok(None);
            }
        }
    }
}

/// Represents the result of processing user input in file selection
/// 
/// This enum clarifies what action should be taken based on user input
/// during the file selection process.
#[derive(Debug)]
enum FileSelectionAction {
    /// Continue browsing in the current or new directory
    ContinueBrowsing,
    
    /// User selected a CSV file successfully
    FileSelected(CsvFileInformation),
    
    /// User chose to quit the selection process
    QuitSelection,
}

/// Information about an item found during directory scanning
/// 
/// This represents either a subdirectory or a file that can be displayed
/// in the file selection interface.
#[derive(Debug, Clone)]
pub struct DirectoryItem {
    /// The name of the item (filename or directory name)
    pub item_name: String,
    
    /// Absolute path to the item
    pub absolute_path: PathBuf,
    
    /// Whether this item is a directory (true) or file (false)
    pub is_directory: bool,
    
    /// File size in bytes (only relevant for files)
    pub file_size_bytes: Option<u64>,
    
    /// Whether this appears to be a CSV file (only relevant for files)
    pub appears_to_be_csv: bool,
}

/// Scans a directory for files and subdirectories suitable for navigation
/// 
/// This function examines a directory and returns information about all
/// accessible items, with special attention to CSV files.
/// 
/// # Arguments
/// * `directory_path` - The directory to scan
/// 
/// # Returns
/// * `RowsAndColumnsResult<Vec<DirectoryItem>>` - List of items found or error
fn scan_directory_for_navigation(directory_path: &Path) -> RowsAndColumnsResult<Vec<DirectoryItem>> {
    let directory_entries = fs::read_dir(directory_path)
        .map_err(|io_error| {
            create_file_system_error(
                &format!("Failed to read directory: {}", directory_path.display()),
                io_error
            )
        })?;
    
    let mut directory_items = Vec::new();
    
    for entry_result in directory_entries {
        let directory_entry = entry_result.map_err(|io_error| {
            create_file_system_error(
                &format!("Failed to process directory entry in: {}", directory_path.display()),
                io_error
            )
        })?;
        
        let entry_path = directory_entry.path();
        let entry_metadata = match directory_entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                // Skip items we can't get metadata for
                continue;
            }
        };
        
        let item_name = match entry_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue, // Skip items without names
        };
        
        // Skip hidden files and directories (starting with .)
        if item_name.starts_with('.') {
            continue;
        }
        
        let is_directory = entry_metadata.is_dir();
        let file_size_bytes = if is_directory { None } else { Some(entry_metadata.len()) };
        
        // Check if this appears to be a CSV file
        let appears_to_be_csv = !is_directory && 
            (item_name.to_lowercase().ends_with(".csv") || 
             item_name.to_lowercase().ends_with(".tsv"));
        
        directory_items.push(DirectoryItem {
            item_name,
            absolute_path: entry_path,
            is_directory,
            file_size_bytes,
            appears_to_be_csv,
        });
    }
    
    // Sort items: directories first, then CSV files, then other files
    directory_items.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,  // Directories first
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // Same type, prioritize CSV files among files
                match (a.appears_to_be_csv, b.appears_to_be_csv) {
                    (true, false) => std::cmp::Ordering::Less,  // CSV files before other files
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.item_name.to_lowercase().cmp(&b.item_name.to_lowercase()),
                }
            }
        }
    });
    
    Ok(directory_items)
}

/// Displays directory contents in a numbered list format
/// 
/// This function shows the current directory and all items in an FF-style
/// numbered list, making it easy for users to select items.
/// 
/// # Arguments
/// * `current_directory` - The directory being displayed
/// * `directory_items` - The items to display
fn display_directory_contents(current_directory: &Path, directory_items: &[DirectoryItem]) {
    println!("Current directory: {}", current_directory.display());
    println!();
    
    if directory_items.is_empty() {
        println!("  (No accessible files or directories)");
    } else {
        for (index, item) in directory_items.iter().enumerate() {
            let item_number = index + 1;
            
            if item.is_directory {
                println!("  {:2}. [DIR]  {}", item_number, item.item_name);
            } else if item.appears_to_be_csv {
                let size_info = format_file_size(item.file_size_bytes.unwrap_or(0));
                println!("  {:2}. [CSV]  {} ({})", item_number, item.item_name, size_info);
            } else {
                let size_info = format_file_size(item.file_size_bytes.unwrap_or(0));
                println!("  {:2}. [FILE] {} ({})", item_number, item.item_name, size_info);
            }
        }
    }
    
    println!();
}

/// Formats file size in human-readable format
/// 
/// # Arguments
/// * `size_bytes` - File size in bytes
/// 
/// # Returns
/// * `String` - Human-readable size (e.g., "1.2 MB", "456 KB", "12 B")
fn format_file_size(size_bytes: u64) -> String {
    const KILOBYTE: u64 = 1_024;
    const MEGABYTE: u64 = KILOBYTE * 1_024;
    const GIGABYTE: u64 = MEGABYTE * 1_024;
    
    if size_bytes >= GIGABYTE {
        format!("{:.1} GB", size_bytes as f64 / GIGABYTE as f64)
    } else if size_bytes >= MEGABYTE {
        format!("{:.1} MB", size_bytes as f64 / MEGABYTE as f64)
    } else if size_bytes >= KILOBYTE {
        format!("{:.1} KB", size_bytes as f64 / KILOBYTE as f64)
    } else {
        format!("{} B", size_bytes)
    }
}

/// Processes user input during file selection
/// 
/// This function interprets user commands and navigates accordingly,
/// handling directory changes, file selection, and quit commands.
/// 
/// # Arguments
/// * `user_input` - The trimmed user input string
/// * `directory_items` - Available items in current directory
/// * `current_directory` - Mutable reference to current directory path
/// 
/// # Returns
/// * `RowsAndColumnsResult<FileSelectionAction>` - Action to take based on input
fn process_user_file_selection_input(
    user_input: &str,
    directory_items: &[DirectoryItem],
    current_directory: &mut PathBuf,
) -> RowsAndColumnsResult<FileSelectionAction> {
    
    match user_input {
        "" => {
            // Empty input means refresh current directory
            Ok(FileSelectionAction::ContinueBrowsing)
        }
        
        "q" | "quit" => {
            println!("File selection cancelled.");
            Ok(FileSelectionAction::QuitSelection)
        }
        
        "b" | "back" => {
            // Go to parent directory
            if let Some(parent_directory) = current_directory.parent() {
                *current_directory = parent_directory.to_path_buf();
                println!("Moving to parent directory...");
            } else {
                println!("Already at root directory.");
            }
            Ok(FileSelectionAction::ContinueBrowsing)
        }
        
        _ => {
            // Try to parse as a number for item selection
            match user_input.parse::<usize>() {
                Ok(selection_number) if selection_number > 0 && selection_number <= directory_items.len() => {
                    let selected_item = &directory_items[selection_number - 1];
                    
                    if selected_item.is_directory {
                        // Navigate into the selected directory
                        *current_directory = selected_item.absolute_path.clone();
                        println!("Entering directory: {}", selected_item.item_name);
                        Ok(FileSelectionAction::ContinueBrowsing)
                    } else if selected_item.appears_to_be_csv {
                        // Selected a CSV file
                        let csv_file_info = create_csv_file_information(selected_item)?;
                        println!("Selected CSV file: {}", selected_item.item_name);
                        Ok(FileSelectionAction::FileSelected(csv_file_info))
                    } else {
                        // Selected a non-CSV file
                        println!("Selected file is not a CSV file. Please select a .csv or .tsv file.");
                        Ok(FileSelectionAction::ContinueBrowsing)
                    }
                }
                
                Ok(_) => {
                    println!("Invalid selection number. Please enter a number between 1 and {}.", directory_items.len());
                    Ok(FileSelectionAction::ContinueBrowsing)
                }
                
                Err(_) => {
                    println!("Invalid input. Enter a number, 'b' for back, 'q' to quit, or Enter to refresh.");
                    Ok(FileSelectionAction::ContinueBrowsing)
                }
            }
        }
    }
}

/// Creates CsvFileInformation from a DirectoryItem
/// 
/// This function validates the selected file and creates a comprehensive
/// information structure about the CSV file for further processing.
/// 
/// # Arguments
/// * `directory_item` - The selected directory item representing a CSV file
/// 
/// # Returns
/// * `RowsAndColumnsResult<CsvFileInformation>` - CSV file information or error
fn create_csv_file_information(directory_item: &DirectoryItem) -> RowsAndColumnsResult<CsvFileInformation> {
    let file_size_bytes = directory_item.file_size_bytes.unwrap_or(0);
    let file_size_human_readable = format_file_size(file_size_bytes);
    
    // Test if the file is readable
    let is_readable = test_file_readability(&directory_item.absolute_path);
    
    let filename_only = directory_item.item_name.clone();
    
    Ok(CsvFileInformation {
        absolute_file_path: directory_item.absolute_path.clone(),
        filename_only,
        file_size_bytes,
        file_size_human_readable,
        is_readable,
    })
}

/// Tests whether a file can be opened for reading
/// 
/// # Arguments
/// * `file_path` - Path to the file to test
/// 
/// # Returns
/// * `bool` - True if file appears readable, false otherwise
fn test_file_readability(file_path: &Path) -> bool {
    match fs::File::open(file_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }
    
    #[test]
    fn test_csv_file_information_structure() {
        let test_info = CsvFileInformation {
            absolute_file_path: PathBuf::from("/test/data.csv"),
            filename_only: "data.csv".to_string(),
            file_size_bytes: 1024,
            file_size_human_readable: "1.0 KB".to_string(),
            is_readable: true,
        };
        
        assert_eq!(test_info.filename_only, "data.csv");
        assert_eq!(test_info.file_size_bytes, 1024);
        assert!(test_info.is_readable);
    }
    
    #[test]
    fn test_directory_item_structure() {
        let test_item = DirectoryItem {
            item_name: "test.csv".to_string(),
            absolute_path: PathBuf::from("/test/test.csv"),
            is_directory: false,
            file_size_bytes: Some(2048),
            appears_to_be_csv: true,
        };
        
        assert!(!test_item.is_directory);
        assert!(test_item.appears_to_be_csv);
        assert_eq!(test_item.file_size_bytes, Some(2048));
    }
}
