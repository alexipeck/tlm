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
Coming soon

## Usage
```
Usage:
  tlm.exe [OPTIONS]

tlm: Transcoding Library Manager

Optional arguments:
  -h,--help             Show this help message and exit
  --disable-print       Disables printing by default. Specific types of print
                        can be enabled on top of this
  --print-content       Enable printing content
  --print-shows         Enable printing shows
  --print-general       Enable printing general debug information
  --config CONFIG       Set a custom config path
  --min-severity MIN_SEVERITY
                        Set a minimum severity (debug, info, warning, error,
                        critical)
```
