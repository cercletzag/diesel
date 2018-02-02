use proc_macro2::Span;

pub trait DiagnosticShim {
    fn error<T: Into<String>>(self, msg: T) -> Diagnostic;
    fn warning<T: Into<String>>(self, msg: T) -> Diagnostic;
}

#[cfg(feature = "nightly")]
impl DiagnosticShim for Span {
    fn error<T: Into<String>>(self, msg: T) -> Diagnostic {
        self.unstable().error(msg)
    }

    fn warning<T: Into<String>>(self, msg: T) -> Diagnostic {
        self.unstable().warning(msg)
    }
}

#[cfg(not(feature = "nightly"))]
impl DiagnosticShim for Span {
    fn error<T: Into<String>>(self, msg: T) -> Diagnostic {
        Diagnostic::error(msg)
    }

    fn warning<T: Into<String>>(self, msg: T) -> Diagnostic {
        Diagnostic::warning(msg)
    }
}

#[cfg(feature = "nightly")]
pub use proc_macro::Diagnostic;

#[cfg(not(feature = "nightly"))]
pub struct Diagnostic {
    message: String,
    level: Level,
}

#[cfg(not(feature = "nightly"))]
impl Diagnostic {
    fn error<T: Into<String>>(msg: T) -> Self {
        Diagnostic {
            message: msg.into(),
            level: Level::Error,
        }
    }

    fn warning<T: Into<String>>(msg: T) -> Self {
        Diagnostic {
            message: msg.into(),
            level: Level::Warning,
        }
    }

    pub fn help(mut self, msg: &str) -> Self {
        self.message.push_str("\n");
        self.message.push_str(msg);
        self
    }

    pub fn note(self, msg: &str) -> Self {
        self.help(msg)
    }

    pub fn emit(self) {
        match self.level {
            Level::Error => panic!("{}", self.message),
            Level::Warning => println!("{}", self.message),
        }
    }
}

#[cfg(not(feature = "nightly"))]
enum Level {
    Warning,
    Error,
}
