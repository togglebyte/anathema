use russh::keys::ssh_key;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Ssh(russh::Error),
    SshKey(ssh_key::Error),
    Runtime(anathema_runtime::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Ssh(e) => write!(f, "SSH error: {}", e),
            Self::SshKey(e) => write!(f, "SSH key error: {}", e),
            Self::Runtime(e) => write!(f, "Runtime error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Ssh(e) => Some(e),
            Self::SshKey(e) => Some(e),
            Self::Runtime(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<russh::Error> for Error {
    fn from(err: russh::Error) -> Self {
        Self::Ssh(err)
    }
}

impl From<ssh_key::Error> for Error {
    fn from(err: ssh_key::Error) -> Self {
        Self::SshKey(err)
    }
}

impl From<anathema_runtime::Error> for Error {
    fn from(err: anathema_runtime::Error) -> Self {
        Self::Runtime(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
