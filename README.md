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
cache_directories = []
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
root_directories = ["/home/ryan/tlmfiles", "/srv/data"]
cache_directories = []
```
The list of root directories are the roots of media collection, in this example
I have two and all paths under them will be scanned for media files. This
can take a significant amount of time if run on a whole disk with many files
so I recommend setting many roots instead but it will work either way

The port is the port used for websocket connections, currently it just
accepts simple commands (import, process, hash) and can be tested using
pretty much any web socket tool but I use [websocat](https://github.com/vi/websocat) for testing

Allowed extensions define the file extensions that files must have to be
imported. In future this will be determined by ffmpeg instead to get all
files that it can handle

Ignored paths ignores and path you wish


## Usage
```
Usage:
  tlm [OPTIONS]

tlm: Transcoding Library Manager

Optional arguments:
  -h,--help             Show this help message and exit
  --disable-print,--no-print
                        Disables printing by default. Specific types of print
                        can be enabled on top of this
  --print-generic       Enable printing generic
  --print-shows         Enable printing shows
  --print-episodes      Enable printing episodes
  --print-general       Enable printing general debug information
  --config,-c CONFIG    Set a custom config path
  --enable-timing       Enable program self-timing
  --timing-threshold,--timing-cutoff TIMING_THRESHOLD
                        Threshold for how slow a timed event has to be in order
                        to print
  --whitelist-generic-output
                        Whitelist all output from generic, whitelisting a type
                        will cause it to print regardless of other limiting
                        flags
  --whitelist-show-output
                        Whitelist all output from shows, whitelisting a type
                        will cause it to print regardless of other limiting
                        flags
  --disable-input,--no-input
                        Don't accept any inputs from the user (Testing only
                        will be removed later)
  --port,-p PORT        Overwrite the port set in the config
```
