// src/error_types_module.rs

/// Error handling types for the rows_and_columns CSV analysis system
///
/// This module defines comprehensive error types that cover all possible failure
/// scenarios in CSV processing, data storage, statistical analysis, and TUI rendering.
/// All errors implement proper error propagation and user-friendly messaging.

use std::fmt;
use std::io;

/// Primary error type for all rows_and_columns operations
///
/// This enum covers all categories of errors that can occur during CSV processing,
/// data storage operations, statistical calculations, and TUI dashboard rendering.
/// Each variant provides specific context for the type of failure encountered.
#[derive(Debug)]
pub enum RowsAndColumnsError {
    /// File system operations failed (reading, writing, directory creation)
    FileSystemError {
        /// Description of the file system operation that failed
        operation_description: String,
        /// The underlying I/O error from the standard library
        source_error: io::Error,
    },

    /// CSV file parsing or processing errors
    CsvProcessingError {
        /// Description of the CSV operation that failed
        csv_operation_description: String,
        /// Line number where error occurred (if applicable)
        csv_line_number: Option<usize>,
        /// Column name or index where error occurred (if applicable)
        csv_column_identifier: Option<String>,
    },

    /// General configuration or setup errors
    ConfigurationError {
        /// Description of the configuration problem
        configuration_issue_description: String,
    },
}

impl fmt::Display for RowsAndColumnsError {
    /// Formats error messages for user-friendly display
    ///
    /// # Arguments
    /// * `formatter` - The formatter to write the error message to
    ///
    /// # Returns
    /// * `fmt::Result` - Success or failure of the formatting operation
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowsAndColumnsError::FileSystemError {
                operation_description,
                source_error
            } => {
                write!(
                    formatter,
                    "File system operation failed: {} - Underlying error: {}",
                    operation_description, source_error
                )
            }

            RowsAndColumnsError::CsvProcessingError {
                csv_operation_description,
                csv_line_number,
                csv_column_identifier
            } => {
                let line_info = match csv_line_number {
                    Some(line_num) => format!(" at line {}", line_num),
                    None => String::new(),
                };

                let column_info = match csv_column_identifier {
                    Some(col_id) => format!(" in column '{}'", col_id),
                    None => String::new(),
                };

                write!(
                    formatter,
                    "CSV processing failed: {}{}{}",
                    csv_operation_description, line_info, column_info
                )
            }

            RowsAndColumnsError::ConfigurationError {
                configuration_issue_description
            } => {
                write!(
                    formatter,
                    "Configuration error: {}",
                    configuration_issue_description
                )
            }
        }
    }
}

impl std::error::Error for RowsAndColumnsError {
    /// Returns the underlying source of this error, if any
    ///
    /// # Returns
    /// * `Option<&(dyn std::error::Error + 'static)>` - The source error or None
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RowsAndColumnsError::FileSystemError { source_error, .. } => Some(source_error),
            _ => None,
        }
    }
}

/// Converts IO errors into RowsAndColumnsError with context
///
/// # Arguments
/// * `io_error` - The I/O error to convert
///
/// # Returns
/// * `RowsAndColumnsError` - The converted error with context
impl From<io::Error> for RowsAndColumnsError {
    fn from(io_error: io::Error) -> Self {
        RowsAndColumnsError::FileSystemError {
            operation_description: "I/O operation failed".to_string(),
            source_error: io_error,
        }
    }
}

/// Helper function to create file system errors with context
///
/// # Arguments
/// * `operation_description` - Description of the file operation that failed
/// * `io_error` - The underlying I/O error
///
/// # Returns
/// * `RowsAndColumnsError` - A properly contextualized file system error
pub fn create_file_system_error(
    operation_description: &str,
    io_error: io::Error
) -> RowsAndColumnsError {
    RowsAndColumnsError::FileSystemError {
        operation_description: operation_description.to_string(),
        source_error: io_error,
    }
}

/// Helper function to create CSV processing errors with context
///
/// # Arguments
/// * `csv_operation_description` - Description of the CSV operation that failed
/// * `csv_line_number` - Optional line number where error occurred
/// * `csv_column_identifier` - Optional column name or index where error occurred
///
/// # Returns
/// * `RowsAndColumnsError` - A properly contextualized CSV processing error
pub fn create_csv_processing_error(
    csv_operation_description: &str,
    csv_line_number: Option<usize>,
    csv_column_identifier: Option<String>
) -> RowsAndColumnsError {
    RowsAndColumnsError::CsvProcessingError {
        csv_operation_description: csv_operation_description.to_string(),
        csv_line_number,
        csv_column_identifier,
    }
}

/// Helper function to create configuration errors with context
///
/// # Arguments
/// * `configuration_issue_description` - Description of the configuration problem
///
/// # Returns
/// * `RowsAndColumnsError` - A properly contextualized configuration error
pub fn create_configuration_error(configuration_issue_description: &str) -> RowsAndColumnsError {
    RowsAndColumnsError::ConfigurationError {
        configuration_issue_description: configuration_issue_description.to_string(),
    }
}

/// Type alias for Results that use RowsAndColumnsError
///
/// This makes function signatures more readable throughout the codebase.
pub type RowsAndColumnsResult<T> = Result<T, RowsAndColumnsError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test error display formatting for CSV processing errors
    #[test]
    fn test_csv_processing_error_display() {
        let csv_error = RowsAndColumnsError::CsvProcessingError {
            csv_operation_description: "Invalid column count".to_string(),
            csv_line_number: Some(42),
            csv_column_identifier: Some("customer_name".to_string()),
        };

        let error_string = csv_error.to_string();
        assert!(error_string.contains("Invalid column count"));
        assert!(error_string.contains("42"));
        assert!(error_string.contains("customer_name"));
    }

    /// Test error display formatting for file system errors
    #[test]
    fn test_file_system_error_display() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let fs_error = RowsAndColumnsError::FileSystemError {
            operation_description: "Creating directory".to_string(),
            source_error: io_error,
        };

        let error_string = fs_error.to_string();
        assert!(error_string.contains("Creating directory"));
        assert!(error_string.contains("Permission denied"));
    }

    /// Test helper function for creating file system errors
    #[test]
    fn test_create_file_system_error_helper() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = create_file_system_error("Reading CSV file", io_error);

        match error {
            RowsAndColumnsError::FileSystemError {
                operation_description,
                source_error: _,
            } => {
                assert_eq!(operation_description, "Reading CSV file");
            }
            _ => panic!("Expected FileSystemError"),
        }
    }

    /// Test helper function for creating configuration errors
    #[test]
    fn test_create_configuration_error_helper() {
        let error = create_configuration_error("Invalid TOML format");

        match error {
            RowsAndColumnsError::ConfigurationError {
                configuration_issue_description,
            } => {
                assert_eq!(configuration_issue_description, "Invalid TOML format");
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }
}
