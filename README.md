# Logz
Logz is a command line utility that is used to view log files.

## Features
- Pre-define applications by name to list log files in the registered directory for viewing
- View a single log file by location and navigate the log using arrow keys or vim keybinds.
- Pretty print JSON formatted logs
- Color highlighting to help indicate log level.

## Roadmap

V0.1 (MVP):
- ~~A CLI tool where a user enters a file path to tail~~
- A terminal ui that displays the log output in real time.
- ~~It only tails the log and prints out the messages as they arrive in the log file.~~

V0.2:
- Open and view a single text based log file.
- Entire file is viewable with no lag even for log files via scrolling.
- Uses vim motions like h, j, k, l, gg, G, ctrl + d, and ctrl + u for navigating log files.

V0.3:
- Add persistence via json/toml/yaml or sqlite for app names
- Logz application add "myapp" "./application" (register a new app)
- When app is chosen, user is given a list log files to choose from in the tui
- To select, user uses j,k or up arrow, down arrow, and presses enter to select the log file.
- To go back to the list of files, user presses backspace or esc

V0.4:
- Add JSON colorizing and pretty printing
- Allow users to add and manage apps from the TUI instead of just the cli
- From app log directory, going back takes users back to the app listing.
