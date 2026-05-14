# `CS2 GSI Audio Events`

This program runs an HTTP server receiving requests from
[Counterstrike's Gamestate Integration](https://developer.valvesoftware.com/wiki/Counter-Strike:_Global_Offensive_Game_State_Integration)
and triggers things when certain events have happened.

At the moment those "things" are

- Bomb planted countdown timer (once the bomb has been planted a voice will announce "[30, 20, 15, 10, 5] seconds" until
  detonation)
- The same but for the remaining time per round

You will run this program in a terminal.

## Setup

Stop the game, clone this repository and copy the `gamestate_integration_events.cfg` from the repo into your CS game's
config folder.

Depending on your setup the config folder is usually located at
`Steam/steamapps/common/Counter-Strikg Global Offensive/game/csgo/cfg/`

This file tells Counterstrike to send it's Gamestate data to the local webserver we are spinning up in the next step.
Now, you have two options to run it

## Option 1: Build and run from source

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

## Option 2: Bravely run main.exe directly

I've added a pre-compiled `main.exe` to the repository (runs on my Win11). Simply open a terminal, navigate to the
repository's root folder, then execute `.\main.exe` ...

```bash
> .\main.exe
2026-05-14T14:48:45.645007Z  INFO main: Starting server...
2026-05-14T14:48:45.646034Z  INFO main: Server Running
```
