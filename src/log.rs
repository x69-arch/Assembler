use colored::Colorize;

pub enum Log {
    Warning {
        origin: String,
        line: usize,
        message: String,
    },
    
    Error {
        origin: String,
        line: usize,
        message: String,
    }
}

impl Log {
    pub fn is_error(&self) -> bool { matches!(self, Log::Error{..}) }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (origin, line, message) = match self {
            Log::Warning { origin, line, message } => {
                write!(f, "{}", "Warning: ".yellow().bold())?;
                (origin, line, message)
            },
            Log::Error { origin, line, message } => {
                write!(f, "{}", "Error: ".red().bold())?;
                (origin, line, message)
            }
        };
        write!(f, "{}:{}: {}", origin, line + 1, message)
    }
}
