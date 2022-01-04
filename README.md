# Transcoding Library Manager(tlm)
[![dependency status](https://deps.rs/repo/github/alexipeck/tlm/status.svg)](https://deps.rs/repo/github/alexipeck/tlm)
## Description
Tlm is currently a cross platform command line application for maintaining a database of video files
for transcoding with ffmpeg in an automated way. Video file lists are maintained from a set of user
defined directories and a list of allowed extensions.

## Dependencies
* [ffmpeg](https://ffmpeg.org/)
* [Diesel](https://diesel.rs/)
* [Postgres](https://www.postgresql.org/)
* [Mediainfo](https://mediaarea.net/en/MediaInfo)

## Installation
Currently the setup for tlm isn't very complicated, you'll need postgres running and diesel setup to interact with it.
Mediainfo will also need to be installed and in path
A guide to setting up diesel can be found [here](https://diesel.rs/guides/getting-started.html).<br/>

Make sure to run ```diesel database setup``` before the first run to populate postgres with tables<br/>

By default tlm will
create a file called .tlm_config where it is first run with some default parameters but the config location can be changed
via the ```--config /path/to/config``` argument. You can run the binary and generate the file or create it in advance from 
the below template which uses toml

```toml
allowed_extensions = ["mp4", "mkv", "webm", "avi"]
ignored_paths = [".recycle_bin"]

[tracked_directories]
root_directories = ["/path/to/directory1", "/path/to/directory2"]
cache_directory = ""
```

## Configuration
On the first run a default configuration file will be created in the users
config directory. This is determined by the [directories](https://docs.rs/directories/4.0.1/directories/) crate. Logs for the program will be stored in a folder here with one file per day
It is written in [toml](https://toml.io) and looks like this

```toml
port = 8888
allowed_extensions = ["mp4", "mkv", "webm"]
ignored_paths = [".recycle_bin"]

[tracked_directories]
root_directories = ["C:\\Users\\Alexi Peck\\Desktop\\tlm\\test_files\\"]
cache_directory = "C:\\Users\\ALEXIP~1\\AppData\\Local\\Temp\\"
```
The list of root directories are the roots of media collection, in this example
I have two and all paths under them will be scanned for media files.
This should be run in specific directories or network shares, such as those dedicated to media libraries, rather than running from the root of a drive, etc.

The port is the port used for websocket connections, currently it can receive simple commands such as (import, process, hash, generate_profiles, output_tracked_paths, display_workers, run_completeness_check) from pretty much any web socket tool but I use [websocat](https://github.com/vi/websocat) for testing. The server communicates with workers with that same port, but with encoded messages (can't be tested with websocat, etc).

Allowed extensions define the file extensions that any given file must have in order to be imported.
In future this will be limited by ffmpeg instead, allowing all the codecs it can handle

Ignored paths ignores and path you wish

## Dev Environment
### Test Files
```
https://mega.nz/file/SVhXjawB#RCghmkPiH894GM4bf52SdO-jYx47xDbo-qsFzDfSjEc
```

### Update crates and cargo tools
```
cargo update
```

### Code auto-formatting
```
cargo fmt
```

### Run commands (from working directory)
```
cargo build --release
```
```
diesel database reset
```
```
.\target\release\tlm-server.exe
```
```
.\target\release\tlm-worker.exe
```
```
websocat ws://127.0.0.1:8888
```

### Database changes
```
diesel migration generate name_of_database_migration
```
```
diesel migration run
```

## Usage
```
Usage:
  tlm [OPTIONS]

tlm: Transcoding Library Manager

Optional arguments:
  -h,--help                     Show this help message and exit
  -c,--config CONFIG            Set a custom config path
  --disable-input,--no-input    Don't accept any inputs from the user
  --fsro,
  --fs_read_only,
  --file_system_read_only,      Don't overwrite any media files in the file system
  --port,-p PORT                Overwrite the port set in the config
  -s,--silent                   Disables output
```
