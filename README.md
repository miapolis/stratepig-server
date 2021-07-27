# Stratepig Server

This is the server code for my game Stratepig, 
which I'm making public mainly for two reasons:
- I'm rather new to Rust and Tokio so I'd like to have this be open source so others can give feedback more easily
- I've been working on Stratepig since May of 2020, and I'd like to have something to show since the client code is garbage

I am aware I do a lot of bad things here, such as 
waste time converting client IDs to strings when sending them to clients, 
and this is because there was an original server implementation written in C# 
that used UUIDs and I'm too lazy to change things on the client as of right now.

Honestly I'm sort of surprised that I got everything working with this 
server + room architecture that isn't really safe in Rust, 
and I'd love to hear feedback and criticism on my implementation as much as possible.

Overall, this was a learning experience more than anything else, 
and I will probably not be writing TCP servers in low level languages anytime soon.
However, I do now have a blazingly fast server as far as I am aware.

## Technical Details

| Crate                                | Description |
:--------------------------------------| :------------
| [stratepig_cli](stratepig_cli)       | Parses arguments at the start of the application and contains the `CliConfig` struct
| [stratepig_core](stratepig_core)     | Manages packet de/serialization, network buffers, mio event loop, and other low level things
| [stratepig_game](stratepig_game)     | Contains all game-related objects, constants, and functions for paths             
| [stratepig_macros](stratepig_macros) | Includes the `#[server_packet(id)` and `#[client_packet(id)]` procedular macros for stratepig_core
| [stratepig_server](stratepig_server) | The main crate that ties everything together. Includes most server and game logic

#### stratepig_core
The core crate is mostly a mirror of [grubbnet](https://github.com/Dooskington/grubbnet),
with some minor improvements and using a newer version of [mio](https://github.com/tokio-rs/mio).
I also added macros to help with defining packets in [stratepig_macros](stratepig_macros).

#### stratepig_updater
Elixir update server for client downloads and possibly other things in the future.
