# css-demo-to-bhoptimer-replay
Problems:
- not perfect
- probably breaks if there's multiple players in a server
- doesn't produce accurate `!keys` / replay-frame `buttons`
    - it also sets `IN_JUMP` for every frame
- broken `m_fFlags` stuff...

Example usage:
`cargo run -- "[U:1:123]" "bhop_tranquility.replay" 55.466 350 6300`

**TODO:**
- fix `m_fFlags` stuff...
    - might have to properly iterate entities / use LocalPlayer and actually read the class member variables rather than using Cheat Engine to find random offsets in memory...
- check for `-insecure`?
