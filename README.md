<h1 align="center">mplayer-server</h1>

A [DBus](https://www.freedesktop.org/wiki/Software/dbus/#whatisd-bus) music player server wirtten in [Rust](https://www.rust-lang.org)

# Thanks to
- [Loft-rs](https://github.com/Serial-ATA/lofty-rs)
- [audiotag](https://github.com/TianyiShi2001/audiotags)
- [Zbus](https://github.com/dbus2/zbus)
- [Serder](https://github.com/serde-rs/serde)
- [Kira](https://github.com/tesselode/kira)
> and many more 

# Methods
|Name       |Type  |Signature|Result/Value|Flags|
|-----------|------|---------|----------- |-----|
|End        |method| -       |b           |  -  |
|Metadata   |method| -       |(sss(ays))  |  -  |
|Pause      |method| -       |b           |  -  |
|Play       |method| s       |b           |  -  |
|Playing    |method| -       |(s(tu)sss)  |  -  |
|Resume     |method| -       |b           |  -  |
|Seek       |method| d       |b           |  -  |
|Status     |method| -       |s           |  -  |
|Timer      |method| -       |s           |  -  |
|ToggleMute |method| -       |b           |  -  |
|Volume     |method| d       |b           |  -  |
# Descriptions
- *End*
> _Descriptoin_: Terminate the playing music
> - _Result Kind_: True if no panic happened
- Metadata
> _Description_: Returns the playing music metadata
> - _Result Kind_: Deserialized [Metadata](./src/server.rs) type
- Pause
> _Description_: Pauses the playing music
> - _Result Kind_: True if no panic happened
- Play
> _Description_: Plays the music from file path
> - _Result Kind_: True if no panic happened
- Resume
> _Description_: Resumes the playing music
> - _Result Kind_: True if no panic happened
- Seek
> _Description_: seeks the player by the given duration relative to the current playing time
>> - if state is playing :
>> 	    - seek by the give nduration
>> - if state is Stopping it:
>> 	    - plays the preivously played song
>> 	    - seeks by the given duration
>> - if state is pausing it:
>> 	    - resumes the currently playing song 
>> 	    - seeks by the given duration
> - _Result Kind_: True if no panic happened
- Status
> _Description_: Returns the player status
> - _Result Kind_: Follows the following schema:
>> Status: <Playing|Pausing|Paused>\nAudio Path: <Path>\nVolume: <value between 0 and 1>
- Timer
> _Description_: played duration over the the total duration of the music 
> - _Result Kind_: full length/played duration as a string
- ToggleMute
> _Description_: Toggles audio
> - _Result Kind_: True if no panic happened
- Volume
> _Description_: Change the volume of the player
> - _Result Kind_: True if no panic happened
- Play
> _Descriptoin_: Plays the music from the file path
> - _Result Kind_: True if no panic happened
