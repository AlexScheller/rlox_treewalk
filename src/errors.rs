use std::fmt;
use std::process;

use crate::source_file;

#[derive(Clone)]
pub struct ErrorDescription {
    pub subject: Option<String>,
    pub location: source_file::SourceSpan,
    pub description: String,
}

impl fmt::Display for ErrorDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ErrorDescription {
            subject,
            location,
            description,
        } = self;
        if let Some(subject_value) = subject {
            write!(
                f,
                "[line: {}, col: {}] Error ({}): {}",
                location.start.line, location.start.column, description, subject_value
            )
        } else {
            write!(
                f,
                "[line: {}, col: {}] Error ({})",
                location.start.line, location.start.column, description
            )
        }
    }
}

// Right now these aren't really used... Rather than a log of heterogenous errors, each system
// maintains it's own contextual log of error descriptions. Keeping in case I discover a good reason
// they need to be differentiated later.
pub enum Error {
    Scanning(ErrorDescription),
    Parsing(ErrorDescription),
    // Runtime(ErrorDescription)
}

// TODO: This feels wrong.
impl Error {
    pub fn description(&self) -> ErrorDescription {
        let description = match self {
            Error::Scanning(description) => description,
            Error::Parsing(description) => description,
        };
        description.clone()
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
    pub errors: Vec<ErrorDescription>,
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
    pub fn push(&mut self, error: ErrorDescription) {
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

fn print_error_log(log: &ErrorLog) {
    for error in log.errors.iter() {
        println!("{}", error.to_string());
    }
}

pub fn report_and_exit(code: exitcode::ExitCode, error_log: &ErrorLog) {
    print_error_log(error_log);
    exit_with_code(code);
}
