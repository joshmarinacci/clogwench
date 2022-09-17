mod audio;
mod output;

use core;
use symphonia::core::errors::Error;
use db::{JObj};
use crate::audio::{AudioContext, open_audio_track};
use crate::output::{AudioOutput};

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
    // pub fn play_track(&self, track:&JObj) {
    //     println!("playing a track {:?}",track);
    //     println!("file path is {}",track.data.get("filepath").unwrap());
        //open the audio track to get an AudioContext
        //in a loop
        //get the next packet
        //decode the next packet
        //write it to the audio output
    // }
    pub fn load_track(&self, track:&JObj) -> AudioPlayer {
        let path =  String::from(track.data.get("filepath").unwrap());
        let ctx = open_audio_track(&path);
        let ap = AudioPlayer {
            path: path,
            ctx:ctx,
            running: false
        };
        return ap;
    }
}

pub struct AudioPlayer {
    path:String,
    running:bool,
    ctx:AudioContext,
}

impl AudioPlayer {
    pub fn play(&mut self, audio_output: &mut Option<Box<dyn AudioOutput>>, ) {
        self.running = true;
        loop {
            let packet = match self.ctx.probe_result.format.next_packet() {
                Ok(packet) => packet,
                Err(Error::ResetRequired) => {
                    println!("reset required");
                    unimplemented!()
                }
                Err(err) => {
                    println!("error . end of stream?");
                    break;
                }
            };
            match self.ctx.decoder.decode(&packet) {
                Ok(decoded) => {
                    // Consume the decoded audio samples (see below).
                    // println!("got some samples {}", decoded.frames());
                    if audio_output.is_none() {
                        // println!("trying to open a device");
                        // Get the audio buffer specification. This is a description of the decoded
                        // audio buffer's sample format and sample rate.
                        let spec = *decoded.spec();
                        // println!("spec is {:?}",spec);

                        // Get the capacity of the decoded buffer. Note that this is capacity, not
                        // length! The capacity of the decoded buffer is constant for the life of the
                        // decoder, but the length is not.
                        let duration = decoded.capacity() as u64;
                        // println!("duraction is {}",duration);

                        // Try to open the audio output.
                        audio_output.replace(output::try_open(spec, duration).unwrap());
                    } else {
                        // println!("still open");
                    }
                    if let Some(audio_output) = audio_output {
                        audio_output.write(decoded).unwrap()
                    }
                }
                Err(Error::IoError(_)) => {
                    println!("io error");
                    // The packet failed to decode due to an IO error, skip the packet.
                    continue;
                }
                Err(Error::DecodeError(_)) => {
                    println!("decode error");
                    // The packet failed to decode due to invalid data, skip the packet.
                    continue;
                }
                Err(err) => {
                    // An unrecoverable error occured, halt decoding.
                    println!("{}", err);
                }
            }
        }
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
