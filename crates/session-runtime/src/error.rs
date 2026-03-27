#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunFailureKind {
    Overflow,
    Transient,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunFailure {
    pub kind: RunFailureKind,
    pub message: String,
}

impl RunFailure {
    pub fn overflow(message: impl Into<String>) -> Self {
        Self {
            kind: RunFailureKind::Overflow,
            message: message.into(),
        }
    }

    pub fn transient(message: impl Into<String>) -> Self {
        Self {
            kind: RunFailureKind::Transient,
            message: message.into(),
        }
    }

    pub fn fatal(message: impl Into<String>) -> Self {
        Self {
            kind: RunFailureKind::Fatal,
            message: message.into(),
        }
    }

    pub fn is_overflow(&self) -> bool {
        matches!(self.kind, RunFailureKind::Overflow)
    }
}
