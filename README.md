# Log Viewer RS

A log viewer with terminal UI.

![screenshot](res/screenshot.png "screenshot")

## Build
Install rustup and run:
```sh
cargo run --release
```

## Supported platforms
### Windows

Windows is the main platform that's tested, but this project is expected to work on other platforms with zero to minimal effort.

### Ubuntu
The following packages need to be installed for this project to build on Ubuntu:

```sh
sudo apt-get install librust-atk-dev
sudo apt-get install librust-gdk-sys-dev
```

## Features
- Filtering log entires by multiple comma-separated keywords
- Searching log entries by multiple comma-separated keywords
- Viewing entries combined from multiple log files ordered by log date
- Ability to tail log files in real time
- Copying log entries to clipboard (Windows-only)
- Prettified JSON view for log entries that contain JSON data

## Supported log formats
Only UTF-8 encoded logs are supported currently. The following formats are supported, but more formats can be added per request:
- Windows (MSI) installer logs
- CEF logs
- Multiple log formats from game launchers on Windows (e.g., Steam)

## Key bindings
| Action | Keys |
| ---  | ---     |
| Change the currently active log entry (skipping 5 entries at a time) | `{` / `}` (or `PageUp` / `PageDown`) |
| Change the currently active log entry | `j`/`k` (or down/up arrow keys) |
| Change the currently active tab | `h` / `l` (or left/right arrow keys)  |
| Close the current tab | `x` |
| Copy the selected log entry to clipboard (Windows only) | `c` | 
| Enable/disable tailing | `t` |
| Exit the current view / Remove focus from the currently focused input field | `Esc` / `<C-c>`|
| Filter log entries using multiple comma-separated keywords | `f` |
| Go to the end of the file | `<S-G>` or `End` | 
| Go to the next / previous search match (when no input field is focused) | `n` / `p` | 
| Go to the next / previous search match (when the search input field is focused) | `Up` / `Down` arrow keys | 
| Go to the start of the file | `gg` or `Home` | 
| Search for log entires using multiple comma-separated keywords | `s` |
| Show a file picker to open a new log file in a new tab (only supported in GUI environments) | `o` |

