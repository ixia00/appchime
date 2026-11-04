mod identity;
mod process;
mod watcher;

pub mod catalog;
pub mod window;
pub mod icon;

pub use identity::AppIdentity;
pub use process::{ProcessInfo, ProcessScanner};
pub use watcher::{AppEvent, AppEventKind, Watcher};
