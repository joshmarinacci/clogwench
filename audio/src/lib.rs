mod audio;
mod output;
use db::{JObj};
use crate::audio::{AudioPlayer};

pub struct AudioService {
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
    pub fn load_track(&self, track:&JObj) -> AudioPlayer {
        let path =  String::from(track.data.get("filepath").unwrap());
        return AudioPlayer::load(&path);
    }
}

#[cfg(test)]
mod tests {
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
        // processor.load(None);
        //check that the mimetype is mp3
        // assert_eq!(processor.parsed_mimetype(),"audio/mpeg".to_string());
        //check the current progress
        // assert_eq!(processor.current_time(),0);
        let mut audio_output:Option<Box<dyn output::AudioOutput>> = None;
        processor.play(&mut audio_output);
        //sleep for 2 seconds
        //stop the processor
        // processor.pause();
        //check the current progress
        // assert_eq!(processor.current_time(),2*1000);
        // processor.stop();
        audio.shutdown();
    }
}
