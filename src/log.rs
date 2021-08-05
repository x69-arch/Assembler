use colored::Colorize;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Level {
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
    level: Level,
}

impl Log {
    pub fn new(level: Level, origin: Option<Origin>, message: String) -> Self {
        Self {
            origin,
            message,
            level,
        }
    }
    
    pub fn is_error(&self) -> bool { matches!(self.level, Level::Error) }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.level {
            Level::Warning => write!(f, "{}", "Warning: ".yellow().bold())?,
            Level::Error => write!(f, "{}", "Error: ".red().bold())?,
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
        self.logs.push(Log::new(Level::Warning, self.origin.clone(), message));
    }
    
    pub fn log_error(&mut self, message: String) {
        self.logs.push(Log::new(Level::Error, self.origin.clone(), message));
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
    fn drain_logs(&mut self, logger: &mut Logger) {
        for mut log in self.logs.drain(0..) {
            if log.origin.is_none() {
                log.origin = logger.origin.clone();
            }
            logger.logs.push(log);
        }
    }
    
    pub fn unwrap_logged(mut self, logger: &mut Logger) -> Option<T> {
        self.drain_logs(logger);
        self.result
    }
    
    pub fn if_ok<F: FnOnce(T)>(mut self, logger: &mut Logger, callback: F) {
        self.drain_logs(logger);
        if let Some(result) = self.result {
            callback(result);
        }
    }
    
    pub fn unwrap(self) -> (Option<T>, Vec<Log>) { (self.result, self.logs) }
    
    pub fn is_ok(&self) -> bool { self.result.is_some() }
    pub fn is_err(&self) -> bool { self.result.is_none() }
}
