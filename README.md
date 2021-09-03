# Transcoding Library Manager(tlm)
## Description
Tlm is currently a cross platform command line application for maintaining a database of video files
for transcoding with ffmpeg in an automated way. Video file lists are maintained from a set of user
defined directories and a list of allowed extensions.

## Dependencies
* [ffmpeg](https://ffmpeg.org/)
* [Diesel](https://diesel.rs/)
* [Postgres](https://www.postgresql.org/)

## Installation
Currently the setup for tlm isn't very complicated, you'll need postgres running and diesel setup to interact with it.
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


## Usage
```
Usage:
  ./target/release/tlm [OPTIONS]

tlm: Transcoding Library Manager

Optional arguments:
  -h,--help             Show this help message and exit
  --disable-print,--no-print
                        Disables printing by default. Specific types of print
                        can be enabled on top of this
  --print-generic       Enable printing generic
  --print-shows         Enable printing shows
  --print-general       Enable printing general debug information
  --config,-c CONFIG    Set a custom config path
  --min-severity,--min-verbosity MIN_VERBOSITY
                        Set a minimum severity (debug, info, warning, error,
                        critical)
  --enable-timing       Enable program self-timing
  --timing-threshold,--timing-cutoff TIMING_THRESHOLD
                        Threshold for how slow a timed event has to be in order
                        to print
  --whitelist-content-output
                        Whitelist all output from content, whitelisting a type
                        will cause it to print regardless of other limiting
                        flags
  --whitelist-show-output
                        Whitelist all output from shows, whitelisting a type
                        will cause it to print regardless of other limiting
                        flags
  --disable-input,--no-input
                        Don't accept any inputs from the user (Testing only
                        will be removed later)
```
