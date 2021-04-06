# Hikaru
_Hikaru_ (the crate) is a simple wrapper around the [Chess.com API](https://www.chess.com/news/view/published-data-api) that makes it easy to fetch the game data of a single player.

## Usage
```rust
use hikaru::GameData;

// Check out Hikaru's first game on Chess.com
dbg!(Gamedata::download("hikaru").next())
```
