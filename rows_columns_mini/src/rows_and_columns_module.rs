// src/rows_and_columns_module.rs

/// Primary module for the rows_and_columns CSV analysis and TUI dashboard system
///
/// This module serves as the main entry point for CSV data processing, analysis, and
/// visualization. It manages the binary-relative directory structure for persistent
/// data storage and coordinates all CSV operations through a terminal user interface.
///
/// # Core Responsibilities
/// - Initialize and manage the rows_columns_data/ directory structure
/// - Coordinate CSV file imports and directory-based storage
/// - Provide the main application interface following FF-style patterns
/// - Integrate with file selection and TUI dashboard modules
///
/// # Directory Structure Created
/// ```
/// rows_columns_data/
/// ├── csv_imports/           # Imported CSV datasets
/// └── analysis_cache/        # Computed statistics cache
/// ```
///
/// # Design Philosophy
/// - Binary-executable-relative paths for portable deployment
/// - Persistent directory-based data storage (not temporary)
/// - No pre-loading: on-demand data processing for scalability
/// - Clear error handling with comprehensive user feedback
use std::env;
use std::path::PathBuf;
use std::io::{self, Write};

// Import enhanced CSV analysis capabilities
use super::csv_processor_module::{
    analyze_csv_file_structure_and_types,
    // CsvAnalysisResults,
    perform_enhanced_statistical_analysis,
    // display_enhanced_csv_analysis_results,
    save_analysis_details_to_file,
    save_analysis_summary_to_file,
};

// Import our custom error types for comprehensive error handling
use super::error_types_module::{
    // RowsAndColumnsError,
    RowsAndColumnsResult,
    create_file_system_error,
    create_configuration_error
};

// Import the path management module for binary-relative operations
use super::manage_absolute_executable_directory_relative_paths::{
    make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path,
    get_absolute_path_to_executable_parentdirectory
};
/// Configuration constants for directory structure
///
/// These define the standard directory names used throughout the application
/// for organizing CSV data and analysis results.
const ROWS_COLUMNS_ROOT_DIRECTORY_NAME: &str = "rows_columns_data";
const CSV_IMPORTS_SUBDIRECTORY_NAME: &str = "csv_imports";
const ANALYSIS_CACHE_SUBDIRECTORY_NAME: &str = "analysis_cache";

/// Primary application entry point for rows_and_columns CSV analysis system
///
/// This function handles command line arguments and initializes the application environment.
/// It supports direct CSV file processing via command line arguments, similar to the 'lines' pattern.
///
/// # Command Line Usage
/// * `rows_and_columns` - Interactive mode (future implementation)
/// * `rows_and_columns <csv_file_path>` - Process specific CSV file
/// * `rows_and_columns --help` - Show usage information
///
/// # Returns
/// * `RowsAndColumnsResult<()>` - Success or detailed error information
///
/// # Errors
/// * `RowsAndColumnsError::FileSystemError` - If directory creation or file access fails
/// * `RowsAndColumnsError::ConfigurationError` - If setup validation fails
///
/// # Examples
/// ```bash
/// # Process a specific CSV file
/// rows_and_columns data/customers.csv
///
/// # Show help
/// rows_and_columns --help
/// ```
pub fn run_rows_and_columns_application() -> RowsAndColumnsResult<PathBuf> {
    // Parse command line arguments
    let command_line_arguments: Vec<String> = env::args().collect();

    // // Step 1: Display startup information to user
    // display_application_startup_banner();

    // Step 2: Initialize and verify directory structure
    let directory_paths = initialize_application_directory_structure()?;

    // Step 3: Validate directory setup was successful
    validate_directory_structure_initialization(&directory_paths)?;

    // Step 4: Display success information to user
    // display_directory_setup_success(&directory_paths);

    // Step 5: Process command line arguments
    if command_line_arguments.len() > 1 {
        match command_line_arguments[1].as_str() {
            "--help" | "-h" | "help" => {
                display_usage_help_information();
                return Ok(PathBuf::new());
            }
            _ => {
                // Treat the first argument as a CSV file path
                let csv_file_path = &command_line_arguments[1];
                return rc_analyze_datafile_save_results_to_resultsfile(csv_file_path);
            }
        }
    } else {
        // No command line arguments - start interactive file input
        let csv_file_path_from_qa = interactive_csv_file_path_input()?;
        return rc_analyze_datafile_save_results_to_resultsfile(&csv_file_path_from_qa);
    }
}

/// Displays usage help information for command line interface
///
/// This function shows users how to use the rows_and_columns application
/// with various command line options and file processing modes.
fn display_usage_help_information() {
    println!("USAGE:");
    println!("  rows_and_columns <csv_file_path>     Process a specific CSV file");
    println!("  rows_and_columns --help              Show this help information");
    println!();
    println!("EXAMPLES:");
    println!("  rows_and_columns data/customers.csv");
    println!("  rows_and_columns /home/user/sales_data.csv");
    println!("  rows_and_columns ../reports/quarterly.csv");
    println!();
    println!("FEATURES:");
    println!("  • Directory-based CSV data storage for scalability");
    println!("  • Pandas-style statistical analysis");
    println!("  • ASCII/Unicode TUI charts and visualizations");
    println!("  • Binary-relative path management for portability");
    println!();
}

/// Interactive Q&A to get CSV file path from user
///
/// This function provides a user-friendly interface for selecting a CSV file
/// when no command line arguments are provided. It includes validation and
/// helpful prompts to guide the user.
///
/// # Returns
/// * `RowsAndColumnsResult<String>` - The validated CSV file path or error
///
/// # Errors
/// * `RowsAndColumnsError::FileSystemError` - If file input/validation fails
/// * `RowsAndColumnsError::ConfigurationError` - If user cancels or invalid input
fn interactive_csv_file_path_input() -> RowsAndColumnsResult<String> {
    println!("No CSV file specified. Let's find one to analyze!");
    println!();

    loop {
        // Display helpful instructions
        display_file_input_instructions();

        // Get user input
        print!("Enter CSV file path: ");
        io::stdout().flush().map_err(|io_error| {
            create_file_system_error("Failed to flush stdout for user input", io_error)
        })?;

        let mut user_file_path_input = String::new();
        io::stdin().read_line(&mut user_file_path_input).map_err(|io_error| {
            create_file_system_error("Failed to read user input", io_error)
        })?;

        let trimmed_file_path = user_file_path_input.trim();

        // Handle special commands
        match trimmed_file_path.to_lowercase().as_str() {
            "" => {
                println!("Please enter a file path, or 'quit' to exit.");
                println!();
                continue;
            }
            "quit" | "q" | "exit" => {
                println!("Goodbye!");
                return Err(create_configuration_error("User chose to quit file selection"));
            }
            "help" | "h" | "?" => {
                display_detailed_file_input_help();
                continue;
            }
            _ => {
                // Try to validate the provided file path
                match validate_user_provided_csv_file_path(trimmed_file_path) {
                    Ok(validated_path) => {
                        println!("✓ Found CSV file: {}", validated_path);
                        println!();
                        return Ok(validated_path);
                    }
                    Err(validation_error) => {
                        println!("❌ File issue: {}", validation_error);
                        println!("Please try again, or type 'help' for assistance.");
                        println!();
                        continue;
                    }
                }
            }
        }
    }
}

/// Displays helpful instructions for file input
///
/// This function shows users what kind of input is expected and what
/// commands are available during the file selection process.
fn display_file_input_instructions() {
    println!("📁 File Selection Help:");
    println!("  • Enter the path to your CSV file (absolute or relative)");
    println!("  • Examples:");
    println!("    data/customers.csv");
    println!("    /home/user/reports/sales.csv");
    println!("    ../datasets/analysis_data.csv");
    println!("  • Special commands:");
    println!("    'help' - Show detailed help");
    println!("    'quit' - Exit the application");
    println!();
}

/// Displays detailed help for file input process
///
/// This function provides comprehensive guidance for users who need
/// more help with file path specification and troubleshooting.
fn display_detailed_file_input_help() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Detailed File Input Help");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    println!("FILE PATH FORMATS:");
    println!("  Absolute paths (full path from root):");
    println!("    Linux/Mac: /home/username/data/file.csv");
    println!("    Windows:   C:\\Users\\username\\data\\file.csv");
    println!();

    println!("  Relative paths (from current directory):");
    println!("    Same directory:    file.csv");
    println!("    Subdirectory:      data/file.csv");
    println!("    Parent directory:  ../file.csv");
    println!();

    println!("FILE REQUIREMENTS:");
    println!("  • File must exist and be readable");
    println!("  • File extension should be .csv or .tsv");
    println!("  • File should contain comma-separated values");
    println!();

    println!("TROUBLESHOOTING:");
    println!("  • Check file path spelling and capitalization");
    println!("  • Ensure file permissions allow reading");
    println!("  • Use tab completion in your terminal if available");
    println!("  • Try absolute path if relative path doesn't work");
    println!();

    println!("EXAMPLES OF VALID PATHS:");
    println!("  ./data/customers.csv");
    println!("  ~/Documents/spreadsheet.csv");
    println!("  /tmp/analysis_data.csv");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!();
}

/// Validates a user-provided CSV file path with helpful error messages
///
/// This function checks the file path and provides specific, actionable
/// error messages to help users resolve file access issues.
///
/// # Arguments
/// * `user_file_path` - The file path provided by the user
///
/// # Returns
/// * `Result<String, String>` - Validated path or user-friendly error message
fn validate_user_provided_csv_file_path(user_file_path: &str) -> Result<String, String> {
    // Convert to PathBuf for easier manipulation
    let file_path = PathBuf::from(user_file_path);

    // Check if file exists
    if !file_path.exists() {
        return Err(format!(
            "File not found: '{}'. Please check the path and try again.",
            user_file_path
        ));
    }

    // Check if it's actually a file (not a directory)
    if !file_path.is_file() {
        if file_path.is_dir() {
            return Err(format!(
                "'{}' is a directory, not a file. Please specify a CSV file within this directory.",
                user_file_path
            ));
        } else {
            return Err(format!(
                "'{}' exists but is not a regular file. Please specify a CSV file.",
                user_file_path
            ));
        }
    }

    // Check file extension (with helpful suggestions)
    let file_extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match file_extension.as_deref() {
        Some("csv") | Some("tsv") => {
            // Perfect - this looks like a CSV file
        }
        Some(other_extension) => {
            println!("⚠️  Warning: File extension '{}' is not typical for CSV files.", other_extension);
            println!("   Proceeding anyway, but ensure this contains comma-separated values.");
        }
        None => {
            println!("⚠️  Warning: File has no extension.");
            println!("   Proceeding anyway, but ensure this contains comma-separated values.");
        }
    }

    // Try to convert to absolute path
    let absolute_file_path = match file_path.canonicalize() {
        Ok(abs_path) => abs_path,
        Err(_) => {
            return Err(format!(
                "Cannot resolve absolute path for '{}'. Check file permissions and path validity.",
                user_file_path
            ));
        }
    };

    // Return the absolute path as a string
    Ok(absolute_file_path.to_string_lossy().to_string())
}

/// Modified version of process_csv_file_from_command_line that saves reports to file
///
/// This function performs the same analysis as before but saves results to a TOML
/// report file instead of printing to terminal. The report file is created in the
/// same directory as the source CSV file.
///
/// # Arguments
/// * `csv_file_path_argument` - The CSV file path provided as command line argument
/// * `directory_paths` - The application directory structure for data storage
///
/// # Returns
/// * `RowsAndColumnsResult<()>` - Success or detailed error information
///
/// # Errors
/// * `RowsAndColumnsError::FileSystemError` - If file access or validation fails
/// * `RowsAndColumnsError::CsvProcessingError` - If CSV parsing fails
/// * `RowsAndColumnsError::MetadataError` - If metadata operations fail
pub fn rc_analyze_datafile_save_results_to_resultsfile(
    csv_file_path_argument: &str,
) -> RowsAndColumnsResult<PathBuf> {
    // Step 1: Validate the provided file path
    let csv_file_absolute_path = validate_csv_file_path_from_argument(csv_file_path_argument)?;

    // Step 2: Analyze CSV structure and column types (basic analysis)
    let csv_analysis_results = analyze_csv_file_structure_and_types(&csv_file_absolute_path)?;

    // Step 3: Perform enhanced statistical analysis
    let enhanced_analysis_results = perform_enhanced_statistical_analysis(
        &csv_file_absolute_path,
        &csv_analysis_results
    )?;

    // Step 4: Save summary to report file (creates new file)
    let report_file_path = save_analysis_summary_to_file(
        &csv_file_absolute_path,
        &csv_analysis_results,
        // directory_paths
    )?;

    // Step 5: Append detailed analysis to report file
    save_analysis_details_to_file(
        &report_file_path,
        &enhanced_analysis_results
    )?;

    // Step 6: Notify user of report creation (minimal output to terminal)
    println!("Analysis report here: {}", report_file_path.display());

    // Step 7: Return the report file path
    Ok(report_file_path)
}

/// Validates a CSV file path provided as command line argument
///
/// This function checks if the provided path exists, is accessible, and appears
/// to be a CSV file based on its extension and basic validation.
///
/// # Arguments
/// * `csv_file_path_argument` - The file path string from command line
///
/// # Returns
/// * `RowsAndColumnsResult<PathBuf>` - Absolute path to validated CSV file or error
///
/// # Errors
/// * `RowsAndColumnsError::FileSystemError` - If file doesn't exist or isn't accessible
/// * `RowsAndColumnsError::ConfigurationError` - If file doesn't appear to be CSV
fn validate_csv_file_path_from_argument(csv_file_path_argument: &str) -> RowsAndColumnsResult<PathBuf> {
    // Convert to PathBuf for easier manipulation
    let file_path = PathBuf::from(csv_file_path_argument);

    // Check if file exists
    if !file_path.exists() {
        return Err(create_file_system_error(
            &format!("CSV file does not exist: {}", csv_file_path_argument),
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")
        ));
    }

    // Check if it's actually a file (not a directory)
    if !file_path.is_file() {
        return Err(create_configuration_error(
            &format!("Path exists but is not a file: {}", csv_file_path_argument)
        ));
    }

    // Check file extension suggests CSV format
    let file_extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match file_extension.as_deref() {
        Some("csv") | Some("tsv") => {
            // File appears to be CSV format
        }
        Some(other_extension) => {
            println!("Warning: File extension '{}' is not typical for CSV files.", other_extension);
            println!("         Proceeding anyway, but ensure this is a comma-separated values file.");
            println!();
        }
        None => {
            println!("Warning: File has no extension. Ensure this is a comma-separated values file.");
            println!();
        }
    }

    // Convert to absolute path for consistent handling
    let absolute_file_path = file_path.canonicalize()
        .map_err(|io_error| {
            create_file_system_error(
                &format!("Failed to resolve absolute path for: {}", csv_file_path_argument),
                io_error
            )
        })?;

    Ok(absolute_file_path)
}

// /// Displays application startup banner with version and purpose information
// ///
// /// This provides clear user feedback that the application is starting and
// /// explains its purpose. Follows the minimalist FF-style interface approach.
// fn display_application_startup_banner() {
//     println!("═══════════════════════════════════════════════════════════════");
//     println!("  rows_and_columns - CSV Analysis & TUI Dashboard System");
//     println!("  Version: 1.0.0 | Rust Edition: 2024 | License: MIT");
//     println!("═══════════════════════════════════════════════════════════════");
//     println!("  • Directory-based CSV data storage for scalability");
//     println!("  • Pandas-style statistical analysis");
//     println!("  • ASCII/Unicode TUI charts and visualizations");
//     println!("  • Binary-relative path management for portability");
//     println!("═══════════════════════════════════════════════════════════════");
//     println!();
// }

/// Structure to hold all important directory paths for the application
///
/// This centralizes path management and makes it easy to pass directory
/// information between functions while maintaining type safety.
#[derive(Debug, Clone)]
pub struct ApplicationDirectoryPaths {
    /// Absolute path to the executable's parent directory
    pub executable_parent_directory: PathBuf,

    /// Absolute path to the main rows_columns_data directory
    pub rows_columns_root_directory: PathBuf,

    /// Absolute path to the csv_imports subdirectory
    pub csv_imports_directory: PathBuf,

    /// Absolute path to the analysis_cache subdirectory
    pub analysis_cache_directory: PathBuf,
}

/// Initializes the complete directory structure for the application
///
/// This function creates all necessary directories using binary-relative paths
/// and returns the absolute paths for use throughout the application. It ensures
/// the directory structure is ready for CSV data storage and analysis operations.
///
/// # Returns
/// * `RowsAndColumnsResult<ApplicationDirectoryPaths>` - All directory paths or error
///
/// # Errors
/// * `RowsAndColumnsError::FileSystemError` - If any directory creation fails
///
/// # Directory Structure Created
/// ```
/// executable_directory/
/// └── rows_columns_data/
///     ├── csv_imports/
///     └── analysis_cache/
/// ```
fn initialize_application_directory_structure() -> RowsAndColumnsResult<ApplicationDirectoryPaths> {
    // Get the executable's parent directory for reference
    let executable_parent_directory = get_absolute_path_to_executable_parentdirectory()
        .map_err(|io_error| {
            create_file_system_error(
                "Failed to determine executable parent directory",
                io_error
            )
        })?;

    // Create the main rows_columns_data directory
    let rows_columns_root_directory = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        ROWS_COLUMNS_ROOT_DIRECTORY_NAME
    ).map_err(|io_error| {
        create_file_system_error(
            &format!("Failed to create main directory: {}", ROWS_COLUMNS_ROOT_DIRECTORY_NAME),
            io_error
        )
    })?;

    // Create the csv_imports subdirectory
    let csv_imports_relative_path = format!(
        "{}/{}",
        ROWS_COLUMNS_ROOT_DIRECTORY_NAME,
        CSV_IMPORTS_SUBDIRECTORY_NAME
    );

    let csv_imports_directory = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        &csv_imports_relative_path
    ).map_err(|io_error| {
        create_file_system_error(
            &format!("Failed to create CSV imports directory: {}", csv_imports_relative_path),
            io_error
        )
    })?;

    // Create the analysis_cache subdirectory
    let analysis_cache_relative_path = format!(
        "{}/{}",
        ROWS_COLUMNS_ROOT_DIRECTORY_NAME,
        ANALYSIS_CACHE_SUBDIRECTORY_NAME
    );

    let analysis_cache_directory = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        &analysis_cache_relative_path
    ).map_err(|io_error| {
        create_file_system_error(
            &format!("Failed to create analysis cache directory: {}", analysis_cache_relative_path),
            io_error
        )
    })?;

    // Return the complete directory structure information
    Ok(ApplicationDirectoryPaths {
        executable_parent_directory,
        rows_columns_root_directory,
        csv_imports_directory,
        analysis_cache_directory,
    })
}

/// Validates that the directory structure was created correctly
///
/// This function performs post-creation validation to ensure all directories
/// exist, are accessible, and have the expected properties. It provides an
/// additional safety check after directory creation.
///
/// # Arguments
/// * `directory_paths` - The directory paths to validate
///
/// # Returns
/// * `RowsAndColumnsResult<()>` - Success or validation error
///
/// # Errors
/// * `RowsAndColumnsError::ConfigurationError` - If validation fails
fn validate_directory_structure_initialization(
    directory_paths: &ApplicationDirectoryPaths
) -> RowsAndColumnsResult<()> {
    // Validate executable parent directory
    if !directory_paths.executable_parent_directory.exists() {
        return Err(create_configuration_error(
            "Executable parent directory does not exist after initialization"
        ));
    }

    if !directory_paths.executable_parent_directory.is_dir() {
        return Err(create_configuration_error(
            "Executable parent path exists but is not a directory"
        ));
    }

    // Validate main rows_columns_data directory
    if !directory_paths.rows_columns_root_directory.exists() {
        return Err(create_configuration_error(
            "Main rows_columns_data directory does not exist after creation"
        ));
    }

    if !directory_paths.rows_columns_root_directory.is_dir() {
        return Err(create_configuration_error(
            "Main rows_columns_data path exists but is not a directory"
        ));
    }

    // Validate csv_imports subdirectory
    if !directory_paths.csv_imports_directory.exists() {
        return Err(create_configuration_error(
            "CSV imports directory does not exist after creation"
        ));
    }

    if !directory_paths.csv_imports_directory.is_dir() {
        return Err(create_configuration_error(
            "CSV imports path exists but is not a directory"
        ));
    }

    // Validate analysis_cache subdirectory
    if !directory_paths.analysis_cache_directory.exists() {
        return Err(create_configuration_error(
            "Analysis cache directory does not exist after creation"
        ));
    }

    if !directory_paths.analysis_cache_directory.is_dir() {
        return Err(create_configuration_error(
            "Analysis cache path exists but is not a directory"
        ));
    }

    // All validations passed
    Ok(())
}

// /// Displays success information about directory setup to the user
// ///
// /// This provides clear feedback about what directories were created and where
// /// they are located. Helps users understand the application's file organization.
// ///
// /// # Arguments
// /// * `directory_paths` - The successfully created directory paths
// fn display_directory_setup_success(directory_paths: &ApplicationDirectoryPaths) {

//     println!("  Executable Location:");
//     println!("    {}", directory_paths.executable_parent_directory.display());
//     println!();

//     println!("  Data Storage Root:");
//     println!("    {}", directory_paths.rows_columns_root_directory.display());
//     println!();

//     println!("  CSV Imports Directory:");
//     println!("    {}", directory_paths.csv_imports_directory.display());
//     println!();

//     println!("  Analysis Cache Directory:");
//     println!("    {}", directory_paths.analysis_cache_directory.display());
//     println!("═══════════════════════════════════════════════════════════════");
// }

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the directory structure constants are reasonable
    #[test]
    fn test_directory_constants() {
        // Directory names should not be empty
        assert!(!ROWS_COLUMNS_ROOT_DIRECTORY_NAME.is_empty());
        assert!(!CSV_IMPORTS_SUBDIRECTORY_NAME.is_empty());
        assert!(!ANALYSIS_CACHE_SUBDIRECTORY_NAME.is_empty());

        // Directory names should not contain path separators
        assert!(!ROWS_COLUMNS_ROOT_DIRECTORY_NAME.contains('/'));
        assert!(!ROWS_COLUMNS_ROOT_DIRECTORY_NAME.contains('\\'));
        assert!(!CSV_IMPORTS_SUBDIRECTORY_NAME.contains('/'));
        assert!(!CSV_IMPORTS_SUBDIRECTORY_NAME.contains('\\'));
        assert!(!ANALYSIS_CACHE_SUBDIRECTORY_NAME.contains('/'));
        assert!(!ANALYSIS_CACHE_SUBDIRECTORY_NAME.contains('\\'));
    }

    /// Test the ApplicationDirectoryPaths structure
    #[test]
    fn test_application_directory_paths_structure() {
        // Create a test instance with dummy paths
        let test_paths = ApplicationDirectoryPaths {
            executable_parent_directory: PathBuf::from("/test/exe"),
            rows_columns_root_directory: PathBuf::from("/test/exe/rows_columns_data"),
            csv_imports_directory: PathBuf::from("/test/exe/rows_columns_data/csv_imports"),
            analysis_cache_directory: PathBuf::from("/test/exe/rows_columns_data/analysis_cache"),
        };

        // Verify the structure can be created and accessed
        assert!(test_paths.executable_parent_directory.to_string_lossy().contains("exe"));
        assert!(test_paths.rows_columns_root_directory.to_string_lossy().contains("rows_columns_data"));
        assert!(test_paths.csv_imports_directory.to_string_lossy().contains("csv_imports"));
        assert!(test_paths.analysis_cache_directory.to_string_lossy().contains("analysis_cache"));

        // Test that the structure can be cloned
        let cloned_paths = test_paths.clone();
        assert_eq!(test_paths.executable_parent_directory, cloned_paths.executable_parent_directory);
    }

    /// Test directory initialization logic (without actually creating directories)
    #[test]
    fn test_directory_path_construction() {
        // Test path construction logic
        let csv_imports_path = format!(
            "{}/{}",
            ROWS_COLUMNS_ROOT_DIRECTORY_NAME,
            CSV_IMPORTS_SUBDIRECTORY_NAME
        );

        let analysis_cache_path = format!(
            "{}/{}",
            ROWS_COLUMNS_ROOT_DIRECTORY_NAME,
            ANALYSIS_CACHE_SUBDIRECTORY_NAME
        );

        // Verify paths are constructed correctly
        assert_eq!(csv_imports_path, "rows_columns_data/csv_imports");
        assert_eq!(analysis_cache_path, "rows_columns_data/analysis_cache");
    }
}
