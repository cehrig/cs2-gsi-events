# `CS2 GSI Audio Events`

This program runs an HTTP server receiving requests from
[Counterstrike's Gamestate Integration](https://developer.valvesoftware.com/wiki/Counter-Strike:_Global_Offensive_Game_State_Integration)
and triggers things when certain events have happened.

At the moment those "things" are

- Bomb planted countdown timer (once the bomb has been planted a voice will announce "[30, 20, 15, 10, 5] seconds" until
  detonation)
- The same but for the remaining time per round
- Audio indicator when running low on ammo

You will run this program in a terminal.

## Setup

Stop the game, clone this repository and copy the `gamestate_integration_events.cfg` from the repo into your CS game's
config folder.

Depending on your setup the config folder is usually located at
`Steam/steamapps/common/Counter-Strike Global Offensive/game/csgo/cfg/`

This file tells Counterstrike to send it's Gamestate data to the local webserver we are spinning up in the next step.
Now, you have two options to run it

### Option 1: Build and run from source

If you have [Rust installed](https://rust-lang.org/tools/install/), simply build and run the application in the
repository's source folder. Open a terminal and execute `cargo run --release` ...

```bash
> cargo run --release
    Finished `release` profile [optimized] target(s) in 0.12s
     Running `target\release\main.exe`
2026-05-14T14:42:21.739475Z  INFO main: Starting server...
2026-05-14T14:42:21.740517Z  INFO main: Server Running
```

Now start the game, play a match and you should be hearing audio output once the bomb is planted or the round is about
to end.

This should also run on Linux, but I haven't tested.

### Option 2: Download Release

You can find pre-compiled binaries in the Release section. Simply download the archive for your OS. The archives contain
the binary and a `sound` folder containing audio files. When running the program, make sure to run it from archive's
root.

```bash
> .\main.exe
2026-05-14T14:48:45.645007Z  INFO main: Starting server...
2026-05-14T14:48:45.646034Z  INFO main: Server Running
```

## Settings

There are a couple of settings to customize stuff

```
Usage: main.exe [OPTIONS]

Options:
  -p, --port <PORT>              [default: 30001]
  -i, --ip <IP>                  [default: 127.0.0.1]
  -s, --sound-path <SOUND_PATH>  [default: sounds]
  -d, --debug
  -d, --disable <DISABLE>        [possible values: round-timer, bomb-timer, ammo-low]
  -h, --help                     Print help
  -V, --version                  Print version
```

For example, if you want to disable the ammo low indicator, run the program with
`--disable ammo-low`

## Soundpacks

A default set of sound files is part of the published releases. The application looks for audio files in the `sounds/`
directory relative to where you start it from. You can override that path with the `--sound-path` option.

Also - for the round and bomb announcements - you can add your own wav files. E.g. if you want to have an announcement
60 seconds before the round ends, add a file `60.wav` in the sounds folder.

## Accuracy

CS GSI sends bomb planted
events [randomly delayed](https://www.reddit.com/r/GlobalOffensive/comments/3xah5n/counterstrike_global_offensive_update_for_121715/).
It appears as if this delay is somewhere between 0 and 2 seconds, so the countdown until detonation isn't a 100%
accurate.
If you are playing CT, maybe hurry up when you hear the 10 second announcement :-)
