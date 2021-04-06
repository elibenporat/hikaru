# Hikaru
Hikaru (the library), is a simple wrapper around the chess.com API that makes it easy to download game data.

## Usage
```rust
use hikaru::GameData;

// Check out Hikaru's first game on Chess.com
dbg!(Gamedata::download("hikaru").next())
```
