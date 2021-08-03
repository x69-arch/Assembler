use colored::Colorize;

#[derive(Debug)]
pub enum LogLevel {
    Warning,
    Error,
}

#[derive(Debug, Default, Clone)]
pub struct Origin {
    pub file: String,
    pub line: usize,
}

#[derive(Debug)]
pub struct Log {
    origin: Option<Origin>,
    message: String,
    level: LogLevel,
}

impl Log {
    pub fn new(level: LogLevel, origin: Option<Origin>, message: String) -> Self {
        Self {
            origin,
            message,
            level,
        }
    }
    
    pub fn is_error(&self) -> bool { matches!(self.level, LogLevel::Error) }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.level {
            LogLevel::Warning => write!(f, "{}", "Warning: ".yellow().bold())?,
            LogLevel::Error => write!(f, "{}", "Error: ".red().bold())?,
        };
        match &self.origin {
            Some(origin) => write!(f, "{}:{}: {}", origin.file, origin.line + 1, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    pub origin: Option<Origin>,
    logs: Vec<Log>,
}

impl Logger {
    pub fn new(origin: Option<Origin>) -> Self {
        Self {
            origin,
            logs: Vec::new(),
        }
    }
    
    pub fn log_warning(&mut self, message: String) {
        self.logs.push(Log::new(LogLevel::Warning, self.origin.clone(), message));
    }
    
    pub fn log_error(&mut self, message: String) {
        self.logs.push(Log::new(LogLevel::Error, self.origin.clone(), message));
    }
    
    pub fn is_error(&self) -> bool {
        self.logs.iter().any(Log::is_error)
    }
    
    pub fn into_none<T>(self) -> LoggedResult<T> {
        LoggedResult { result: None, logs: self.logs }
    }
    
    pub fn into_result<T, F: FnOnce() -> T>(self, callback: F) -> LoggedResult<T> {
        let result = if self.is_error() {
            None
        } else {
            Some(callback())
        };
        LoggedResult { result, logs: self.logs }
    }
}

pub struct LoggedResult<T> {
    result: Option<T>,
    logs: Vec<Log>,
}

impl<T> LoggedResult<T> {
    pub fn unwrap(self) -> (Option<T>, Vec<Log>) { (self.result, self.logs) }
    
    pub fn if_ok<F: FnOnce(T)>(self, logger: &mut Logger, callback: F) {
        for mut log in self.logs {
            if log.origin.is_none() {
                log.origin = logger.origin.clone();
            }
            logger.logs.push(log);
        }
        if let Some(result) = self.result {
            callback(result);
        }
    }
}
