use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

const IDLE_CLOSE_AFTER: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(250);

enum Command {
    Play { path: PathBuf, volume: f32 },
    Preview { path: PathBuf, volume: f32 },
    StopPreview,
}

#[derive(Clone)]
pub struct AudioPlayer {
    tx: Sender<Command>,
}

impl AudioPlayer {
    pub fn spawn() -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new().name("audio".into()).spawn(move || run(rx))?;
        Ok(Self { tx })
    }

    pub fn play(&self, path: PathBuf, volume: f32) {
        let _ = self.tx.send(Command::Play { path, volume });
    }

    pub fn preview(&self, path: PathBuf, volume: f32) {
        let _ = self.tx.send(Command::Preview { path, volume });
    }

    pub fn stop_preview(&self) {
        let _ = self.tx.send(Command::StopPreview);
    }
}

pub fn probe(path: &Path) -> Result<Option<Duration>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let decoder = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
    Ok(decoder.total_duration())
}

struct Output {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

fn run(rx: Receiver<Command>) {
    let mut output: Option<Output> = None;
    let mut sinks: Vec<Sink> = Vec::new();
    let mut preview: Option<Sink> = None;
    let mut last_activity = Instant::now();

    loop {
        match rx.recv_timeout(TICK) {
            Ok(Command::Play { path, volume }) => {
                if let Some(sink) = start(&mut output, &path, volume) {
                    sinks.push(sink);
                }
                last_activity = Instant::now();
            }
            Ok(Command::Preview { path, volume }) => {
                if let Some(old) = preview.take() {
                    old.stop();
                }
                preview = start(&mut output, &path, volume);
                last_activity = Instant::now();
            }
            Ok(Command::StopPreview) => {
                if let Some(old) = preview.take() {
                    old.stop();
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        sinks.retain(|s| !s.empty());
        if preview.as_ref().is_some_and(|s| s.empty()) {
            preview = None;
        }
        let busy = !sinks.is_empty() || preview.is_some();
        if busy {
            last_activity = Instant::now();
        } else if output.is_some() && last_activity.elapsed() > IDLE_CLOSE_AFTER {
            output = None;
        }
    }
}

fn start(output: &mut Option<Output>, path: &Path, volume: f32) -> Option<Sink> {
    if output.is_none() {
        match OutputStream::try_default() {
            Ok((stream, handle)) => *output = Some(Output { _stream: stream, handle }),
            Err(e) => {
                log::error!("오디오 장치를 열지 못했습니다: {e}");
                return None;
            }
        }
    }
    let out = output.as_ref()?;

    let source = match File::open(path).map(BufReader::new).map(Decoder::new) {
        Ok(Ok(decoder)) => decoder,
        Ok(Err(e)) => {
            log::error!("소리 파일을 해석하지 못했습니다 {}: {e}", path.display());
            return None;
        }
        Err(e) => {
            log::error!("소리 파일을 열지 못했습니다 {}: {e}", path.display());
            return None;
        }
    };

    match Sink::try_new(&out.handle) {
        Ok(sink) => {
            sink.set_volume(volume.clamp(0.0, 1.0));
            sink.append(source);
            Some(sink)
        }
        Err(e) => {
            log::error!("재생을 시작하지 못했습니다: {e}");
            *output = None;
            None
        }
    }
}
