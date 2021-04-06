//! # Hikaru
//! Hikaru provides Rust bindings to the Chess.com API, specifically for downloading all of a player's games.
//! It is named after Grand Master Hikaru Nakamura (unless he objects, in which case I'll change the name).
//! JSON parsing is done via `serde`; `reqwest` is used to get data from the API.
//!
//! ## How to Use
//! All you have to do is feed Hikaru a list of usernames, and you get back a Vec<[GameData]>
//!
//! ```rust
//! use hikaru::GameData;
//!
//! // Check out Hikaru's first game on Chess.com:
//! dbg!(Gamedata::download("hikaru").next())
//! ```
//!
//! ## Future Plans
//! Create a Stockfish wrapper so that you can analyze all your games. The game data include all the moves made in those games,
//! so this can be fed into the engine for a variety of analyses.

use reqwest::blocking::get;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct GameUrls {
    archives: Vec<String>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeClass {
    Bullet,
    Blitz,
    Rapid,
    Daily,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rules {
    Chess,
    Chess960,
    CrazyHouse,
    ThreeCheck,
    KingOfTheHill,
    Horde,
    BugHouse,
    OddsChess,
}

#[derive(Debug, Deserialize, Serialize)]
struct Game {
    #[serde(rename = "url")]
    game_url: String,
    pgn: Option<String>,
    time_control: String,
    start_time: Option<u32>,
    end_time: u32,
    rated: bool,
    fen: String,
    time_class: TimeClass,
    rules: Rules,
    eco: Option<String>,
    tournament: Option<String>,
    #[serde(rename = "match")]
    team_match: Option<String>,
    white: Player,
    black: Player,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GameResult {
    Win,
    TimeOut,
    CheckMated,
    StaleMate,
    Resigned,
    Agreed,
    Repetition,
    Insufficient,
    Abandoned,
    #[serde(rename = "50move")]
    FiftyMove,
    TimeVsInsufficient,
    KingOfTheHill,
    ThreeCheck,
    BugHousePartnerLose,
    BugHousePartnerWin,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum GameResultWinLose {
    Win,
    Loss,
    Draw,
}

impl From<GameResult> for GameResultWinLose {
    fn from(res: GameResult) -> Self {
        use GameResult::*;
        match res {
            Win | BugHousePartnerWin | KingOfTheHill | ThreeCheck => Self::Win,
            CheckMated | BugHousePartnerLose | Abandoned | Resigned => Self::Loss,
            _ => Self::Draw,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Player {
    username: String,
    rating: u32,
    result: GameResult,
    #[serde(rename = "@id")]
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Games {
    games: Vec<Game>,
}

#[allow(non_snake_case)]
#[derive(Default, Clone, Debug, Serialize)]
struct PGN {
    ECO: String,
    ECO_url: String,
    UTC_date: String,
    UTC_time: String,
    start_date: String,
    start_time: String,
}

impl From<&str> for PGN {
    fn from(pgn: &str) -> Self {
        let mut eco = String::new();
        let mut eco_url = String::new();
        let mut utc_date = String::new();
        for line in pgn.split('\n') {
            if line.starts_with("[ECO \"") {
                eco = line
                    .split(" ")
                    .nth(1)
                    .unwrap_or("")
                    .replace('"', "")
                    .replace(']', "")
                    .into();
            }
            if line.starts_with("[ECOUrl \"") {
                eco_url = line
                    .split(" ")
                    .nth(1)
                    .unwrap_or("")
                    .replace('"', "")
                    .replace(']', "")
                    .into();
            }
            if line.starts_with("[UTCDate \"") {
                utc_date = line
                    .split(" ")
                    .nth(1)
                    .unwrap_or("")
                    .replace('"', "")
                    .replace(']', "")
                    .into();
            }
        }

        Self {
            ECO: eco,
            ECO_url: eco_url,
            UTC_date: utc_date,
            UTC_time: String::new(),
            start_time: String::new(),
            start_date: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameData {
    pub game_url: String,
    pub time_control: String,
    pub start_time: Option<u32>,
    pub end_time: u32,
    pub rated: bool,
    pub fen: String,
    pub time_class: TimeClass,
    pub rules: Rules,
    pub eco_game: Option<String>,
    pub tournament: Option<String>,
    #[serde(rename = "match")]
    pub team_match: Option<String>,
    pub white_rating: u32,
    pub white_username: String,
    pub black_rating: u32,
    pub black_username: String,
    pub eco_pgn: String,
    pub eco_url: String,
    pub result: GameResult,
    pub result_win_lose: GameResultWinLose,
    pub rating: u32,
    pub date: String,
    pub colour: String,
    pub win: f32,
    pub player_username: String,
}

impl GameData {
    pub fn download<'a>(user: &'a str) -> impl Iterator<Item = GameData> + 'a {
        get_game_month_urls(user)
            .into_iter()
            .flat_map(|url| get_games(&url))
            .map(move |game| (game, user).into())
    }
}

impl From<(Game, &str)> for GameData {
    fn from((game, user): (Game, &str)) -> Self {
        let pgn: PGN = game.pgn.as_deref().map(PGN::from).unwrap_or_default();

        let is_white = user == game.white.username;
        let result = if is_white {
            game.white.result
        } else {
            game.black.result
        };
        let result_win_lose = result.into();

        let rating = if is_white {
            game.white.rating
        } else {
            game.black.rating
        };
        let colour = if is_white { "White" } else { "Black" };

        let win = match result_win_lose {
            GameResultWinLose::Win => 1.0,
            GameResultWinLose::Draw => 0.5,
            GameResultWinLose::Loss => 0.0,
        };

        Self {
            game_url: game.game_url,
            time_control: game.time_control,
            start_time: game.start_time,
            end_time: game.end_time,
            rated: game.rated,
            fen: game.fen,
            time_class: game.time_class,
            rules: game.rules,
            eco_game: game.eco,
            tournament: game.tournament,
            team_match: game.team_match,
            result,
            result_win_lose,
            white_rating: game.white.rating,
            white_username: game.white.username,
            black_rating: game.black.rating,
            black_username: game.black.username,
            eco_pgn: pgn.ECO,
            eco_url: pgn.ECO_url,
            rating,
            colour: colour.into(),
            win,
            player_username: user.into(),
            date: pgn.UTC_date,
        }
    }
}

fn get_game_month_urls(user: &str) -> Vec<String> {
    let url = format!("https://api.chess.com/pub/player/{}/games/archives", user);

    let text = get(&url)
        .expect("Didn't get a response")
        .text()
        .expect("Invalid response");

    serde_json::from_str::<GameUrls>(&text)
        .expect("Serde error!")
        .archives
}

fn get_games(url: &str) -> Vec<Game> {
    let games_text = get(url)
        .expect("Didn't get a response")
        .text()
        .expect("Invalid response");
    serde_json::from_str::<Games>(&games_text)
        .expect("Serde error!")
        .games
}
