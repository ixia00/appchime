use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::identity::path_key;
use crate::process::{ProcessInfo, ProcessScanner};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEventKind {
    Launched,
    Quit,
}

#[derive(Clone, Debug)]
pub struct AppEvent {
    pub kind: AppEventKind,
    pub process: ProcessInfo,
}

pub struct Watcher {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Watcher {
    pub fn spawn<F>(interval: Duration, mut on_event: F) -> io::Result<Self>
    where
        F: FnMut(AppEvent) + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();

        let handle = thread::Builder::new().name("app-watch".into()).spawn(move || {
            let mut scanner = ProcessScanner::new();
            let mut previous = group(scanner.scan());

            while !stop_flag.load(Ordering::Relaxed) {
                thread::park_timeout(interval);
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                let current = group(scanner.scan());
                for event in diff(&previous, &current) {
                    on_event(event);
                }
                previous = current;
            }
        })?;

        Ok(Self { stop, handle: Some(handle) })
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

fn group(processes: Vec<ProcessInfo>) -> HashMap<String, ProcessInfo> {
    let mut map: HashMap<String, ProcessInfo> = HashMap::new();
    for p in processes {
        let key = match &p.exe_path {
            Some(path) => path_key(path),
            None => format!("name:{}", p.name.to_lowercase()),
        };
        match map.get(&key) {
            Some(existing) if existing.pid <= p.pid => {}
            _ => {
                map.insert(key, p);
            }
        }
    }
    map
}

fn diff(
    previous: &HashMap<String, ProcessInfo>,
    current: &HashMap<String, ProcessInfo>,
) -> Vec<AppEvent> {
    let launched = current
        .iter()
        .filter(|(k, _)| !previous.contains_key(*k))
        .map(|(_, p)| AppEvent { kind: AppEventKind::Launched, process: p.clone() });
    let quit = previous
        .iter()
        .filter(|(k, _)| !current.contains_key(*k))
        .map(|(_, p)| AppEvent { kind: AppEventKind::Quit, process: p.clone() });
    launched.chain(quit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, path: &str) -> ProcessInfo {
        ProcessInfo { pid, name: path.rsplit('/').next().unwrap().into(), exe_path: Some(path.into()) }
    }

    #[test]
    fn multiple_instances_count_as_one_app() {
        let before = group(vec![]);
        let after = group(vec![p(10, "/a/Browser"), p(11, "/a/Browser"), p(12, "/a/Browser")]);
        let events = diff(&before, &after);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, AppEventKind::Launched);
        assert_eq!(events[0].process.pid, 10);
    }

    #[test]
    fn quit_only_when_last_instance_exits() {
        let a = group(vec![p(10, "/a/Browser"), p(11, "/a/Browser")]);
        let b = group(vec![p(11, "/a/Browser")]);
        assert!(diff(&a, &b).is_empty());
        let c = group(vec![]);
        let events = diff(&b, &c);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, AppEventKind::Quit);
    }

    #[test]
    fn watcher_stops_on_drop() {
        let w = Watcher::spawn(Duration::from_millis(50), |_| {}).unwrap();
        drop(w);
    }
}
