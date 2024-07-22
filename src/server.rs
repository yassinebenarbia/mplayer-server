use event_listener::{Event, Listener};
use kira::{
    manager::{
        AudioManager, AudioManagerSettings, backend::DefaultBackend
    },
    sound::{
        streaming::{
            StreamingSoundData, StreamingSoundSettings, StreamingSoundHandle
        },
        FromFileError, PlaybackState
    }, tween::Tween, clock::{ClockSpeed, ClockHandle}
};
use std::{error::Error, path::Path};

use zbus::{interface, connection};


struct Player{
    path: String,
    audio_manager: AudioManager,
    stream_handler: Option<StreamingSoundHandle<FromFileError>>,
    clock: ClockHandle,
    volume: f64,
    soung_length: f64,
}

#[interface(name = "org.zbus.mplayer1")]
impl Player {
    /// changes the volue of the player
    fn volume(&mut self, amount: u8) -> bool{
        if amount > 100 {
            return false
        }

        let volume = amount as f64 / 100.0;
        self.volume = volume;

        match self.stream_handler.as_mut() {
            Some(handler) => {
                handler.set_volume(volume, Tween::default()).unwrap();
                return true;
            }
            None => return false,
        }
    }

    /// seeks the player by the given duration relative to the current seeker
    /// negative number meens seking backward and vice versa
    fn seek(&mut self, duration: f64) -> bool {
        match self.stream_handler.as_mut() {
            Some(handler) => {
                match handler.state() {
                    PlaybackState::Playing => {
                        handler.seek_by(duration).unwrap();
                        return true;
                    },
                    PlaybackState::Pausing | PlaybackState::Paused => {
                        self.audio_manager.resume(Tween::default()).unwrap();
                        handler.seek_by(duration).unwrap();
                        return true;
                    },
                    PlaybackState::Stopping  | PlaybackState::Stopped => {
                        self.play(self.path.clone());
                        self.seek(duration);
                        return true
                    },
                }
            }
            None => return false,
        }
    }

    fn timer(&mut self) -> String {
        match self.stream_handler.as_mut() {
            Some(handler) => {
                match handler.state() {
                    PlaybackState::Stopping | PlaybackState::Stopped => {
                        return format!("{:.2}/{:.2}", self.soung_length, "0.0")
                    },
                    _ => {
                        return format!("{:.2}/{:.2}",self.soung_length, handler.position())
                    }
                }

            },
            None => {
                return String::from("0.0/0.0");
            },
        }
    }

    /// Shows player information
    fn show(&self) -> String {
        if self.path.is_empty() {
            format!(
                "Audio path: No audio\nVolume: {}\nAudio Status: {:#?}",
                self.volume,
                self.audio_manager.state()
                )
        }else {
            format!(
                "Audio path: {}\nVolume: {}\nAudio manager: {:#?}",
                self.path,
                self.volume,
                self.audio_manager.state()
                )
        }
    }

    /// pauses playing
    fn pause(&mut self) -> bool {
        self.audio_manager.pause(Tween::default()).unwrap();
        self.stream_handler.as_mut().unwrap().pause((Tween::default())).unwrap();
        true
    }

    // Resumes playing, after paused
    fn resume(&mut self) -> bool {
        self.audio_manager.resume(Tween::default()).unwrap();
        self.stream_handler.as_mut().unwrap().resume((Tween::default())).unwrap();
        true 
    }

    /// Terminate playing
    fn end(&mut self) -> bool {
        let _ = match self.stream_handler.as_mut() {
            Some(handler) => {
                handler.stop(Tween::default())
            },
            None => return false,
        };
        true
    }

    /// Plays the audio from the file path
    fn play(&mut self, path: String) -> bool {
        // TODO: check if there is a currently playing song
        let p = Path::new(&path);
        if p.is_dir() {
            eprintln!("Expected a file path, got a directory!\ndirectory: {}", path);
            return false 
        }else {
            self.path = path;
        }
        self.end();

        let mut start_time = self.clock.time();
        self.clock.start().unwrap();
        println!("song path: {}", self.path);
        let mut sound_data = StreamingSoundData::from_file(
            self.path.clone(),
            StreamingSoundSettings::default().start_time(start_time)
            );
        match sound_data {
            Ok(sound_data) => {
                self.soung_length = sound_data.duration().as_secs_f64();
                println!("song duration: {}", sound_data.duration().as_secs());
                start_time.ticks = start_time.ticks.checked_add(
                    sound_data.duration().as_millis() as u64
                    ).unwrap();
                self.stream_handler = Some(self.audio_manager.play(sound_data).unwrap());
                self.stream_handler.as_mut().unwrap().set_volume(self.volume, Tween::default()).unwrap();
            },
            _ => return false
        };

        true
    }

    /// Returns the player status, whichs constructed from
    /// state: <Playing|Pausing|Paused>
    fn status(&self) -> String {
        match self.stream_handler.as_ref() {
            Some(handler) =>{
                match handler.state() {
                    PlaybackState::Stopped => format!("Status: {:#?}", PlaybackState::Stopped),
                    _ => format!("Status: {:#?}\nAudio Path: {}", handler.state(), self.path),
                }
            },
            None => format!("Status: {:#?}", PlaybackState::Stopped),
            // format!("Status: {:#?}\nAudio Path: {}", handler.state(), self.path)
        }
    }
}

pub struct Server {
    dbus_addr: String,
    dbus_interf: String,
    app_name: String,
    done: Event,
    player: Player,
}


pub fn path_from_addr(addr: &String) -> String {
    let mut addr = addr.replace('.', "/");
    addr.insert(0, '/');
    addr
}

impl Server {
    /// screates a new server and connect it to the specified
    /// dbus address
    pub fn new(dbus_addr: String) -> Self {
        let mut manager = AudioManager::<DefaultBackend>::new(
                    AudioManagerSettings::default()
                    ).unwrap();
        let clock =  manager.add_clock(
                    ClockSpeed::TicksPerSecond(1000.0)
                    ).unwrap();
        Server { 
            dbus_interf: path_from_addr(&dbus_addr),
            dbus_addr,
            app_name: String::from("mplayer"),
            done: event_listener::Event::new(),
            player: Player {
                path: String::new(),
                audio_manager: manager,
                stream_handler: None,
                volume: 0.5,
                soung_length: 0.0,
                clock,
            }
        }
    }

    /// TODO: change the default address to an appropriate address
    pub fn default() -> Self {
        let mut manager = AudioManager::<DefaultBackend>::new(
                    AudioManagerSettings::default()
                    ).unwrap();
        let clock = manager.add_clock(
                    ClockSpeed::TicksPerSecond(1000.0)
                    ).unwrap();
        Server {
            dbus_addr: String::from("org.zbus.mplayer"),
            dbus_interf: String::from("/org/zbus/mplayer"),
            app_name: String::from("mplayer"),
            done: event_listener::Event::new(),
            player: Player {
                path: String::new(),
                audio_manager: manager,
                stream_handler: None,
                volume: 0.5,
                soung_length: 0.0,
                clock,
            }
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        let done_listener = self.done.listen();
        let _connection = connection::Builder::session()?
            .name(self.dbus_addr.clone())?
            .serve_at(self.dbus_interf.clone(), self.player)?
            .build()
            .await?;

        done_listener.wait();

        Ok(())
    }

}
