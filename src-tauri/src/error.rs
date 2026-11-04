use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("이미 등록된 앱입니다")]
    DuplicateApp,
    #[error("항목을 찾을 수 없습니다")]
    NotFound,
    #[error("지원하지 않는 소리 파일입니다: {0}")]
    UnsupportedAudio(String),
    #[error("파일이 너무 큽니다: {0}")]
    FileTooLarge(String),
    #[error("앱으로 인식할 수 없습니다: {0}")]
    InvalidApp(String),
    #[error("단축키를 쓸 수 없습니다: {0}")]
    InvalidShortcut(String),
    #[error("잘못된 값입니다: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::DuplicateApp => "duplicateApp",
            Self::NotFound => "notFound",
            Self::UnsupportedAudio(_) => "unsupportedAudio",
            Self::FileTooLarge(_) => "fileTooLarge",
            Self::InvalidApp(_) => "invalidApp",
            Self::InvalidShortcut(_) => "invalidShortcut",
            Self::InvalidInput(_) => "invalidInput",
            Self::Io(_) => "io",
            Self::Other(_) => "other",
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        Self::Other(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
