use event_listener::{Event, Listener};
use serde::ser::Serializer;
use serde::Deserializer;
use lofty::{self, file::{AudioFile, TaggedFileExt}, tag::Accessor};
use kira::{
    manager::{
        AudioManager, AudioManagerSettings, backend::DefaultBackend
    },
    sound::{
        
        streaming::{
            StreamingSoundData, StreamingSoundSettings, StreamingSoundHandle
        },
        FromFileError, PlaybackState
    },
    tween::Tween,
    clock::{ClockSpeed, ClockHandle}
};
use std::{error::Error, path::PathBuf, time::Duration};

use zbus::{interface, connection};
use audiotags;

#[derive(PartialEq, Eq, Debug, Ord, PartialOrd, Clone, Default, serde::Deserialize, serde::Serialize)]
#[derive(zbus::zvariant::Type)]
pub struct Music {
    pub title: String,
    pub length: Duration,
    pub path: PathBuf,
    pub artist: String,
    pub genre: String,
}

enum MutedState {
    Muted, UnMuted
}

impl Music {
    pub fn new(path: PathBuf) -> Self {
        Self {
            title: Self::derive_title_from_path(&path),
            path,
            length: Duration::ZERO,
            artist: String::from("Unknown"),
            genre: String::from("Unknown"),
        }
    }

    fn derive_title_from_path(path: &PathBuf) -> String {
        match lofty::probe::Probe::open(path) {
            Ok(v) =>if let Some(t) = v.read().unwrap().primary_tag() {
                t.title().unwrap_or(path.file_name().unwrap().to_string_lossy()).to_string()
            }else {
                path.file_name().unwrap().to_string_lossy().to_string()
            },
                Err(_) => String::from("Default"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct ImageType(audiotags::MimeType);

impl serde::Serialize for ImageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        let c_as_str = format!("{:?}", self.0);
        serializer.serialize_str(&c_as_str)
    }
}

impl<'de> serde::Deserialize<'de> for ImageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> 
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ImageType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Visitor")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error, 
            {
                if v.contains("Jpeg") {
                    Ok(ImageType(audiotags::MimeType::Jpeg))
                } else if v.contains("Png") {
                    Ok(ImageType(audiotags::MimeType::Png))
                } else if v.contains("Tiff") {
                    Ok(ImageType(audiotags::MimeType::Tiff))
                } else if v.contains("Bmp") {
                    Ok(ImageType(audiotags::MimeType::Bmp))
                } else if v.contains("Gif") {
                    Ok(ImageType(audiotags::MimeType::Gif))
                } else {
                    panic!("Unknown field")
                }
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl zbus::zvariant::Type for ImageType {
    fn signature() -> zbus::zvariant::Signature<'static> {
        zbus::zvariant::Signature::from_str_unchecked("s")
    }
}

#[derive(Debug, Clone, zbus::zvariant::Type, serde::Deserialize, serde::Serialize)]
struct Picture{
    data: Vec<u8>,
    tp: ImageType
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, zbus::zvariant::Type)]
pub struct Metadata {
    title: String, 
    artist: String,
    genre: String,
    cover: Picture,
}

struct Player{
    path: PathBuf,
    audio_manager: AudioManager,
    stream_handler: Option<StreamingSoundHandle<FromFileError>>,
    clock: ClockHandle,
    volume: f64,
    muted_volume: f64,
    muted_state: MutedState,
    song_length: f64,
}

#[interface(name = "org.zbus.mplayerServer")]
impl<'a> Player {
    /// changes the volume of the player, return true if no panic happened
    fn volume(&mut self, amount: f64) -> bool{
        println!("changed volume to: {}", amount);
        if amount > 100.0 || amount < 0.0 {
            return true
        }

        let volume = amount / 100.0;
        self.volume = volume;

        match self.stream_handler.as_mut() {
            Some(handler) => {
                match handler.set_volume(volume, Tween::default()){
                    Ok(_) => return true,
                    Err(_) => return false,
                }
            }
            None => return false,
        }
    }

    /// Toggle volume
    fn toggle_mute(&mut self) -> bool{
        println!("mute toggled!");
        match self.muted_state {
            MutedState::UnMuted => {
                self.muted_volume = self.volume;
                self.volume(0.0);
                self.muted_state = MutedState::Muted;
                return true;
            },
            MutedState::Muted => {
                self.volume = self.muted_volume;
                self.volume(self.volume * 100.0);
                self.muted_state = MutedState::UnMuted;
                return true;
            },
        }
    }

    /// Returns current audio [Metadata]
    fn metadata(&mut self) -> Metadata {
        println!("reading data from file...");
        match audiotags::Tag::new().read_from_path(self.path.clone()) {
            Ok(tag) => {
                println!("extracting data from file...");
                let genre: String = tag.genre().unwrap_or("Unknown").to_string().replace('\0', "");
                let title: String = tag.title().unwrap_or("Unknown").to_string().replace('\0', "");
                let artist: String = tag.artist().unwrap_or("Unknown").to_string().replace('\0', "");
                println!("crating album cover...");
                let cover = tag.album_cover().unwrap_or(audiotags::Picture{
                    mime_type: audiotags::MimeType::Jpeg,
                    data: &[0]
                }).clone();
                println!("getting cover data..");
                let tp: audiotags::MimeType = cover.mime_type; 
                let data: Vec<u8> = cover.data.to_owned();

                Metadata {
                    title,
                    artist,
                    genre,
                    cover: Picture {
                        data,
                        tp: ImageType(tp)
                    },
                }
            },
            Err(_) => {
                Metadata {
                    title: String::from("None"),
                    artist: String::from("None"),
                    genre: String::from("None"),
                    cover: Picture {
                        data: vec![0],
                        tp: ImageType(audiotags::MimeType::Jpeg)
                    },
                }

            },
        }
    }

    /// seeks the player by the given duration relative to the current playing timer
    /// negative number meens seking backward and vice versa
    /// 
    /// - if state is in playing it:
    ///     - seeks by the give nduration
    /// - if state is Stopping it:
    ///     - plays the preivously played song
    ///     - seeks by the given duration
    /// if state is pausing it:
    ///     - resumes the currently playing song 
    ///     - seeks by the given duration
    fn seek(&mut self, duration: f64) -> bool {
        println!("seeking by: {}", duration);
        match self.stream_handler.as_mut() {
            Some(handler) => {
                match handler.state() {
                    PlaybackState::Playing => {
                        match handler.seek_by(duration) {
                            Ok(_) => return true,
                            Err(_) => return false,
                        }
                    },
                    PlaybackState::Stopping  | PlaybackState::Stopped => {
                        self.play(self.path.clone());
                        self.seek(duration);
                        return true
                    },
                    PlaybackState::Pausing | PlaybackState::Paused => {
                        self.resume();
                        match self.stream_handler.as_mut().unwrap().seek_by(duration) {
                            Ok(_) => return true,
                            Err(_) => return false,
                        }
                    }
                }
            }
            None => return false,
        }
    }

    /// Played duration over the the total duration of the song
    /// format: full length / played duration
    fn timer(&mut self) -> String {
        match self.stream_handler.as_mut() {
            Some(handler) => {
                match handler.state() {
                    PlaybackState::Stopping | PlaybackState::Stopped => {
                        return format!("{:.2}/{:.2}", self.song_length, "0.0")
                    },
                    _ => {
                        return format!("{:.2}/{:.2}",self.song_length, handler.position())
                    }
                }

            },
            None => {
                return String::from("0.0/0.0");
            },
        }
    }

    /// Pauses playing the currently playing song, returns true if no panic happened
    fn pause(&mut self) -> bool {
        println!("pausing!");
        if let Ok(_) = self.audio_manager.pause(Tween::default()) {
            if let Ok(_) = self.stream_handler.as_mut().unwrap().pause(Tween::default()) {
                return true

            }else {
                return false
            }
        }
        return false
    }

    /// Resumes playing the currently paused song, returns true if no panic happened
    fn resume(&mut self) -> bool {
        println!("resuming");
        if let Ok(_) = self.audio_manager.resume(Tween::default()) {
            if let Ok(_) = self.stream_handler.as_mut().unwrap().resume(Tween::default()) {
                return true;
            }else {
                return false;
            }
        }
        return false
    }

    /// Terminate playing, returns true if no panic happened
    fn end(&mut self) -> bool {
        println!("stopping!");
        let _ = match self.stream_handler.as_mut() {
            Some(handler) => {
                handler.stop(Tween::default())
            },
            None => return false,
        };
        true
    }

    /// Plays the audio from the file path, returns true if no panic happened
    fn play(&mut self, path: PathBuf) -> bool {
        println!("Playing");
        if path.is_dir() {
            eprintln!("Expected a file path, got a directory!\ndirectory: {:?}", path);
            return false 
        }else {
            self.path = path;
        }
        self.end();

        let mut start_time = self.clock.time();
        self.clock.start().unwrap();
        println!("song path: {:#?}", self.path);
        let sound_data = StreamingSoundData::from_file(
            self.path.clone(),
            StreamingSoundSettings::default().start_time(start_time).volume(self.volume)
        );
        match sound_data {
            Ok(sound_data) => {
                self.song_length = sound_data.duration().as_secs_f64();
                println!("song duration: {}", sound_data.duration().as_secs());
                println!("volume: {}", self.volume);
                start_time.ticks = start_time.ticks.checked_add(
                    sound_data.duration().as_millis() as u64
                ).unwrap();

                self.stream_handler = Some(self.audio_manager.play(sound_data).unwrap());
            },
            _ => return false
        };
        true
    }

    /// Returns the player status:
    /// - status: <Playing|Pausing|Paused|Stopping|Stopped>
    /// - path: PATH
    /// - volume: between 0 and 1
    fn status(&self) -> String {
        match self.stream_handler.as_ref() {
            Some(handler) =>{
                let state = handler.state();
                format!("Status: {:#?}\nAudio Path: {:#?}\nVolume: {}", state, self.path, self.volume)
            },
            None => format!("Status: {:#?}\nAudio Path: {:#?}\nVolume: {}", PlaybackState::Stopped, self.path, self.volume),
        }
    }

    /// gets the currently playing [Music]
    pub async fn playing(&self) -> Music {
        let path = self.path.clone().into();
        let res = lofty::probe::Probe::open(&path);
        match res {
            Ok(probe)=>{
                if let Ok(mut x) = probe.read() {
                    let properties = x.properties();
                    let length = properties.duration();
                    if let Some(tag) = x.primary_tag_mut() {
                        let title = tag.title().unwrap_or(std::borrow::Cow::from("Unknown")).to_string();
                        let artist = tag.artist().unwrap_or(std::borrow::Cow::from("Unknown")).to_string();
                        let genre = tag.genre().unwrap_or(std::borrow::Cow::from("Unknown")).to_string();

                        return Music {
                            title, length, path, artist, genre
                        }
                    }else {
                        return Music {
                            path, length,
                            ..Default::default()
                        }
                    }
                }else {
                    println!("couldn't read the music prob in {:?}, falling to default method.", path);
                    return Music::new(path)
                }
            },
            Err(e) => {
                println!("couldn't create the Music object from {:?}, returning the default object.", path);
                println!("Error: {}", e);
                return Music::new(path);
            } 
        }
    }
}

pub struct Server {
    dbus_addr: String,
    dbus_interf: String,
    done: Event,
    player: Player,
    #[allow(dead_code)]
    app_name: String,
}


#[allow(dead_code)]
fn path_from_addr(addr: &String) -> String {
    let mut addr = addr.replace('.', "/");
    addr.insert(0, '/');
    addr
}

impl Server {
    #[allow(dead_code)]
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
                path: PathBuf::default(),
                audio_manager: manager,
                stream_handler: None,
                volume: 0.5,
                muted_volume: 0.0,
                muted_state: MutedState::UnMuted,
                song_length: 0.0,
                clock,
            }
        }
    }

    pub fn default() -> Self {
        let mut manager = AudioManager::<DefaultBackend>::new(
            AudioManagerSettings::default()
        ).unwrap();
        let clock = manager.add_clock(ClockSpeed::TicksPerSecond(1000.0)).unwrap();
        Server {
            dbus_addr: String::from("org.zbus.mplayer"),
            dbus_interf: String::from("/org/zbus/mplayer"),
            app_name: String::from("mplayer"),
            done: event_listener::Event::new(),
            player: Player {
                path: PathBuf::default(),
                audio_manager: manager,
                stream_handler: None,
                volume: 0.5,
                muted_volume: 0.0,
                muted_state: MutedState::UnMuted,
                song_length: 0.0,
                clock,
            }
        }
    }

    /// Starts the dbus server
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
