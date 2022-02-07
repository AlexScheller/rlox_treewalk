use std::fmt;
use std::process;

use crate::source_file;

pub struct ErrorDescription {
    pub subject: Option<String>,
    pub location: Option<source_file::SourceSpan>,
    pub description: String,
}

// impl fmt::Display for ErrorDescription {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let ErrorDescription {
//             subject,
//             location,
//             description,
//         } = self;
//         if let Some(subject_value) = subject {
//             write!(
//                 f,
//                 "[line: {}, col: {}] Error ({}): {}",
//                 location.start.line, location.start.column, description, subject_value
//             )
//         } else {
//             write!(
//                 f,
//                 "[line: {}, col: {}] Error ({})",
//                 location.start.line, location.start.column, description
//             )
//         }
//     }
// }

pub enum ErrorKind {
    Scanning,
    Parsing,
    Runtime,
}

pub struct Error {
    pub kind: ErrorKind,
    pub description: ErrorDescription,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind_string = match self.kind {
            ErrorKind::Scanning | ErrorKind::Parsing => String::from("Syntax"),
            ErrorKind::Runtime => String::from("Runtime"),
        };

        let location_string = if let Some(location_value) = self.description.location {
            format!(
                "[line: {}, col: {}]",
                location_value.start.line, location_value.start.column
            )
        } else {
            String::from("")
        };

        let subject_string = if let Some(subject_value) = &self.description.subject {
            format!(": {}", subject_value)
        } else {
            String::from("")
        };

        write!(
            f,
            "{} {} Error ({}){}",
            location_string, kind_string, self.description.description, subject_string
        )
    }
}
// pub enum Error {
//     Scanning(ErrorDescription),
//     Parsing(ErrorDescription),
// }

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let Error::Syntax(description) = self;
// 		match self {
// 			Error::Scanning(description) => write!(f, "{}", description)
// 		}
//     }
// }

pub struct ErrorLog {
    pub errors: Vec<Error>,
}

impl ErrorLog {
    pub fn new() -> Self {
        ErrorLog { errors: Vec::new() }
    }
    // pub fn log(
    //     &mut self,
    //     location: source_file::SourceSpan,
    //     subject: &str,
    //     description: &str,
    // ) -> &Self {
    //     self.errors.push(ErrorDescription {
    //         subject: Some(String::from(subject)),
    //         location,
    //         description: String::from(description),
    //     });
    //     self
    // }
    pub fn push(&mut self, error: Error) {
        self.errors.push(error);
    }
    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

// Should this really be implemented as an actual `fmt::Display`?
// impl fmt::Display for ErrorLog {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let mut result = String::new();
//         for error in self.errors.iter() {
//             result.push_str(&format!("{}\n", error.to_string()).to_string());
//         }
//         write!(f, "{}", result)
//     }
// }

pub trait ErrorLoggable {
    fn error_log(&self) -> &ErrorLog;
}

pub fn exit_with_code(code: exitcode::ExitCode) {
    process::exit(code);
}

// pub fn exit_on_error(code: exitcode::ExitCode, error_log: &ErrorLog) {
//     println!("{}", error_log);
//     exit_with_code(code);
// }

pub fn print_error_log(log: &ErrorLog) {
    for error in log.errors.iter() {
        println!("{}", error.to_string());
    }
}

pub fn report_and_exit(code: exitcode::ExitCode, error_log: &ErrorLog) {
    print_error_log(error_log);
    exit_with_code(code);
}
