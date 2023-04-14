use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source, StreamError};
use rodio::source::{Buffered, SamplesConverter};
use anyhow::Result;
use tracing::error;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum SoundEffects {
    StartWork,
    StartShortBreak,
    StartLongBreak,
    Pause,
    Unpause,
    Skip,
    StopFinished,
    StopUnfinished,
    Secret,
}

impl SoundEffects {
    fn path(self) -> &'static str {
        match self {
            SoundEffects::StartWork => "owo",
            SoundEffects::StartShortBreak => "owo",
            SoundEffects::StartLongBreak => "owo",
            SoundEffects::Pause => "owo",
            SoundEffects::Unpause => "owo",
            SoundEffects::Skip => "owo",
            SoundEffects::StopFinished => "owo",
            SoundEffects::StopUnfinished => "owo",
            SoundEffects::Secret => "sfx/secret.mp3",
        }
    }
}

pub struct AudioManager {
    stream_handle: OutputStreamHandle,
    cache: RefCell<HashMap<SoundEffects, ExternalSample>>,
}

type ExternalSample = Buffered<SamplesConverter<Decoder<BufReader<File>>, f32>>;

impl AudioManager {
    pub fn new() -> Result<Self, StreamError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        // yes we leak the OutputStream but it's probably fine
        Box::leak(Box::new(stream));
        Ok(Self {
            stream_handle,
            cache: RefCell::new(HashMap::new())
        })
    }

    pub fn play(&self, sound: SoundEffects) -> Result<()>  {
        let sample = match self.cache.borrow_mut().entry(sound) {
            Entry::Occupied(entry) => {
                entry.get().clone()
            }
            Entry::Vacant(entry) => {
                let buf = BufReader::new(File::open(sound.path())?);
                let decoder = Decoder::new(buf)?;
                entry.insert(decoder.convert_samples::<f32>().buffered()).clone()
            }
        };

        self.stream_handle.play_raw(sample)?;

        Ok(())
    }

    pub fn play_logged(&self, sound: SoundEffects) {
        if let Err(e) = self.play(sound) {
            error!("Failed to play audio: {e}");
        }
    }
}
