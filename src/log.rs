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


pub struct LoggedResult<T> {
    result: Option<T>,
    logs: Vec<Log>,
}

impl<T> LoggedResult<T> {
    pub fn new() -> Self {
        Self {
            result: None,
            logs: Vec::new()
        }
    }
    
    pub fn logs(&self) -> &[Log] { &self.logs }
    
    pub fn log_warning(&mut self, message: String) {
        self.logs.push(Log::new(LogLevel::Warning, None, message));
    }
    pub fn log_error(&mut self, message: String) {
        self.result = None;
        self.logs.push(Log::new(LogLevel::Error, None, message));
    }
    
    pub fn push_log(&mut self, log: Log) {
        if log.is_error() {
            self.result = None;
        }
        self.logs.push(log);
    }
    
    pub fn map_origin(mut self, origin: Origin) -> Self {
        for log in &mut self.logs {
            if log.origin.is_none() {
                // TODO: clone every iteration except for the last one
                log.origin = Some(origin.clone());
            }
        }
        self
    }
    
    pub fn take_log_and<T2, F: FnOnce(T)>(self, result: &mut LoggedResult<T2>, callback: F) {
        result.logs.extend(self.logs);
        if let Some(val) = self.result {
            callback(val);
        }
    }
    
    pub fn is_error(&self) -> bool {
        self.logs.iter().any(Log::is_error)
    }
    
    pub fn return_value<F>(mut self, callback: F) -> Self
        where F: FnOnce() -> T
    {
        if !self.is_error() {
            self.result = Some(callback());
        }
        self
    }
    
    pub fn value(self) -> Option<T> {
        self.result
    }
}













#[macro_export]
macro_rules! log {
    ($kind:ident, $origin:expr, $line:expr, $msg:expr) => {
        crate::log::Log::new(
            crate::log::LogLevel::$kind,
            Some(crate::log::Origin { file: $origin.to_owned(), line: $line }),
            format!($msg)
        )
    };
    ($kind:ident, $origin:expr, $line:expr, $msg:expr, $($params:expr),+) => {
        crate::log::Log::new(
            crate::log::LogLevel::$kind,
            Some(crate::log::Origin { file: $origin.to_owned(), line: $line }),
            format!($msg, $($params),+)
        )
    };
}
