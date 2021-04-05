# Hikaru

Hikaru (the library), is a simple wrapper around the chess.com API that makes it easy to download game data.

## Usage

```rust
use hikaru::GameData;

let user_names = vec!["hikaru","GMHikaruOnTwitch"];
let games = GameData::download(user_names);
 
// Check out Hikaru's first game on Chess.com:
dbg!(&games[0]);
```
