
//! # CSV Inspection Tool
//! 
//! This module provides functions for basic inspection of CSV files
//! including header counting, row counting, and displaying CSV data
//! in simple TUI tables.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::env;
use std::process;

/// Parses command line arguments and executes the appropriate CSV inspection function.
///
/// This function handles the CLI commands, validates arguments, and calls the
/// specific inspection functions based on user input.
///
/// # Commands
///
/// * `--describe <file_path>` - Show column count, header names, and row count
/// * `--row <row_index> <file_path>` - Display a specific row with headers
///
/// # Returns
///
/// * `()` - The function exits the process on error
pub fn csv_inspection_main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Parse the command-line arguments
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }
    
    // Process based on the command argument
    match args[1].as_str() {
        "--describe" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path");
                print_usage(&args[0]);
                process::exit(1);
            }
            
            let file_path = &args[2];
            
            // Validate file path exists
            if !Path::new(file_path).exists() {
                eprintln!("Error: File '{}' does not exist", file_path);
                process::exit(1);
            }
            
            // Describe the CSV file
            match describe_csv(file_path) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error describing CSV file: {}", e);
                    process::exit(1);
                }
            }
        },
        "--row" => {
            // Need at least 4 arguments for --row command
            if args.len() < 4 {
                eprintln!("Error: Missing row index or file path");
                print_usage(&args[0]);
                process::exit(1);
            }
            
            // Parse row index
            let row_index = match args[2].parse::<usize>() {
                Ok(idx) => idx,
                Err(_) => {
                    eprintln!("Error: Row index must be a non-negative integer");
                    process::exit(1);
                }
            };
            
            let file_path = &args[3];
            
            // Validate file path exists
            if !Path::new(file_path).exists() {
                eprintln!("Error: File '{}' does not exist", file_path);
                process::exit(1);
            }
            
            // Display the specified row
            match display_csv_row(file_path, row_index) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error displaying row: {}", e);
                    process::exit(1);
                }
            }
        },
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

/// Prints the tool's usage instructions.
///
/// # Arguments
///
/// * `program_name` - The name of the program
fn print_usage(program_name: &str) {
    eprintln!("Usage:");
    eprintln!("  {} --describe <csv_file_path>", program_name);
    eprintln!("  {} --row <row_index> <csv_file_path>", program_name);
    eprintln!("\nExample:");
    eprintln!("  {} --describe data.csv", program_name);
    eprintln!("  {} --row 5 data.csv", program_name);
}

/// Represents a CSV file's structure and content.
struct CsvData {
    /// The column headers from the CSV file
    headers: Vec<String>,
    /// The rows of data, each containing a vector of string values
    rows: Vec<Vec<String>>,
}

/// Reads a CSV file and returns its headers and content.
///
/// # Arguments
///
/// * `file_path` - Path to the CSV file to be read
///
/// # Returns
///
/// * `Result<CsvData, io::Error>` - The parsed CSV data or an error
fn read_csv_file(file_path: impl AsRef<Path>) -> Result<CsvData, io::Error> {
    // Open the file
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    // Prepare data structures
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    
    // Process each line
    for (line_index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        
        // Parse the current line into fields
        let fields = parse_csv_line(&line);
        
        if line_index == 0 {
            // First line is headers
            headers = fields;
        } else {
            // Subsequent lines are data
            rows.push(fields);
        }
    }
    
    Ok(CsvData { headers, rows })
}

/// Parses a line of CSV text into separate fields.
///
/// # Arguments
///
/// * `line` - A line of CSV text to parse
///
/// # Returns
///
/// * `Vec<String>` - Vector of parsed fields
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    
    for ch in line.chars() {
        match ch {
            '"' => {
                // Toggle quoted state
                in_quotes = !in_quotes;
            },
            ',' if !in_quotes => {
                // End of field if comma outside quotes
                fields.push(current_field.trim().to_string());
                current_field = String::new();
            },
            _ => {
                // Add any other character to current field
                current_field.push(ch);
            }
        }
    }
    
    // Add the last field
    fields.push(current_field.trim().to_string());
    
    fields
}

/// Describes a CSV file's structure and outputs a summary.
///
/// # Arguments
///
/// * `file_path` - Path to the CSV file to describe
///
/// # Returns
///
/// * `Result<(), io::Error>` - Success or an I/O error
pub fn describe_csv(file_path: impl AsRef<Path>) -> Result<(), io::Error> {
    // Read the CSV file
    let csv_data = read_csv_file(file_path)?;
    
    // Get header and row counts
    let header_count = csv_data.headers.len();
    let row_count = csv_data.rows.len();
    
    // Print header information as a TUI table
    println!("CSV Structure Summary:");
    println!("=====================");
    println!("Number of columns: {}", header_count);
    println!("Number of data rows: {}", row_count);
    println!();
    
    // Calculate maximum width needed for each column
    let mut column_widths = Vec::new();
    for header in &csv_data.headers {
        column_widths.push(header.len());
    }
    
    // Print headers table
    println!("Column Headers:");
    println!("==============");
    print_headers_table(&csv_data.headers, &column_widths);
    
    Ok(())
}

/// Prints a table showing the headers.
///
/// # Arguments
///
/// * `headers` - Vector of header strings
/// * `column_widths` - Vector of column widths for formatting
fn print_headers_table(headers: &[String], column_widths: &[usize]) {
    // Print top border
    print_horizontal_border(column_widths);
    
    // Print header row
    print!("| ");
    for (i, header) in headers.iter().enumerate() {
        let width = column_widths[i];
        print!("{:<width$} | ", header, width = width);
    }
    println!();
    
    // Print bottom border
    print_horizontal_border(column_widths);
}

/// Displays a specific row from a CSV file with headers.
///
/// # Arguments
///
/// * `file_path` - Path to the CSV file
/// * `row_index` - Index of the row to display (0-based for data rows, not counting header)
///
/// # Returns
///
/// * `Result<(), io::Error>` - Success or an I/O error
pub fn display_csv_row(file_path: impl AsRef<Path>, row_index: usize) -> Result<(), io::Error> {
    // Read the CSV file
    let csv_data = read_csv_file(file_path)?;
    
    // Check if the requested row index is valid
    if row_index >= csv_data.rows.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Row index {} out of range (max: {})", row_index, csv_data.rows.len() - 1)
        ));
    }
    
    // Get the row data
    let row_data = &csv_data.rows[row_index];
    
    // Calculate column widths (max of header and value width)
    let mut column_widths = Vec::new();
    for (i, header) in csv_data.headers.iter().enumerate() {
        let header_width = header.len();
        let value_width = if i < row_data.len() { row_data[i].len() } else { 0 };
        column_widths.push(header_width.max(value_width));
    }
    
    // Print table
    println!("Row {} Data:", row_index);
    println!("===========");
    
    // Print top border
    print_horizontal_border(&column_widths);
    
    // Print header row
    print!("| ");
    for (i, header) in csv_data.headers.iter().enumerate() {
        let width = column_widths[i];
        print!("{:<width$} | ", header, width = width);
    }
    println!();
    
    // Print separator
    print_horizontal_border(&column_widths);
    
    // Print value row
    print!("| ");
    for (i, _) in csv_data.headers.iter().enumerate() {
        let width = column_widths[i];
        let value = if i < row_data.len() { &row_data[i] } else { "" };
        print!("{:<width$} | ", value, width = width);
    }
    println!();
    
    // Print bottom border
    print_horizontal_border(&column_widths);
    
    Ok(())
}

/// Helper function to print horizontal borders for tables.
///
/// # Arguments
///
/// * `column_widths` - Vector of column widths
fn print_horizontal_border(column_widths: &[usize]) {
    print!("+");
    for width in column_widths {
        print!("-{}-+", "-".repeat(*width));
    }
    println!();
}
