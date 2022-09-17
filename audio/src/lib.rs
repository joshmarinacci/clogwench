mod audio;
mod output;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use db::{JObj};
use crate::audio::{AudioCommand, AudioPlayer};
use crate::output::AudioOutput;

pub struct AudioService {
}

pub struct AudioPlayerProxy {
    pub path: String,
    sender:Sender<AudioCommand>,
    pub handler: JoinHandle<()>,
}

impl AudioPlayerProxy {
    pub fn make(path: &str) -> AudioPlayerProxy {
        let (sender, receiver):(Sender<AudioCommand>, Receiver<AudioCommand>) = channel();
        let pth = String::from(path);
        let handler = thread::spawn(move || {
            let mut player = AudioPlayer::load(&pth);
            let mut audio_output:Option<Box<dyn AudioOutput>> = None;
            player.start(&mut audio_output, receiver);
        });
        return AudioPlayerProxy {
            path: String::from(path),
            sender,
            handler,
        }

    }
    pub fn play(&mut self) {
        self.sender.send(AudioCommand::Play()).unwrap();
    }
    pub fn pause(&mut self) {
        self.sender.send(AudioCommand::TogglePlayPause).unwrap();
    }
    pub fn stop(&mut self) {
        self.sender.send(AudioCommand::Quit).unwrap();
    }
}

impl AudioService {
    pub fn make() -> AudioService {
        AudioService {
        }
    }
    pub(crate) fn init(&mut self) {
    }
    pub(crate) fn shutdown(&self) {
    }
    pub fn load_track(&self, track:&JObj) -> AudioPlayerProxy {
        let path =  String::from(track.data.get("filepath").unwrap());
        AudioPlayerProxy::make(&path)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use db::{JDB, JObj};
    use crate::{AudioService, output};

    #[test]
    fn play_mp3() {
        let mut jdb = JDB::make_empty();
        let mut song = JObj::make();
        song.data.insert("type".to_string(), "song".to_string());
        song.data.insert("title".to_string(), "Catch Me I'm Falling".to_string());
        song.data.insert("artist".to_string(), "Pretty Poison".to_string());
        song.data.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        song.data.insert("filepath".to_string(), "./in_your_eyes.mp3".to_string());
        let song = jdb.process_add(song);


        let mut audio = AudioService::make();
        audio.init();
        //get the audio processor reference
        let mut processor = audio.load_track(&song);

        thread::sleep(Duration::from_millis(1000));
        processor.play();
        thread::sleep(Duration::from_millis(1000));
        processor.pause();
        thread::sleep(Duration::from_millis(1000));
        processor.play();
        thread::sleep(Duration::from_millis(1000));
        processor.stop();
        //check the current progress
        // assert_eq!(processor.current_time(),2*1000);
        // processor.stop();
        audio.shutdown();
    }
}
