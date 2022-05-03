// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

pub use std::io::ErrorKind;

#[macro_export]
macro_rules! make_other_err {
    ($($arg:tt)+) => {{
        Error::new(
            ErrorKind::Other,
            format!("{}", format_args!($($arg)+)),
        )
    }};
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub messages: Vec<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, msg: impl ToString) -> Self {
        let mut msgs = Vec::new();
        let msg_string = msg.to_string();
        if !msg_string.is_empty() {
            msgs.push(msg_string);
        }
        Error {
            kind,
            messages: msgs,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A manual impl to reduce the noise of frequently empty fields.
        let mut builder = f.debug_struct("Error");

        builder.field("kind", &self.kind);

        if !self.messages.is_empty() {
            builder.field("messages", &self.messages);
        }

        builder.finish()
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            kind: err.kind(),
            messages: vec![err.to_string()],
        }
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::new(ErrorKind::Other, err.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Error::new(ErrorKind::Other, err.to_string())
    }
}
