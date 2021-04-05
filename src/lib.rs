//! # Hikaru
//! 
//! Hikaru provides Rust bindings to the Chess.com API, specifically for downloading all of a player's games. It is named after Grand Master Hikaru Nakamura (unless he objects, in which case I'll change the name). 
//! JSON parsing is done via SereE; reqwest is used to get data from the API.
//! 
//! ## How to Use
//! 
//! All you have to do is feed Hikaru a list of usernames, and you get back a Vec<[GameData]>
//! 
//! ```rust
//! use hikaru::GameData;
//! 
//! let user_names = vec!["hikaru","GMHikaruOnTwitch"];
//! let games = GameData::download(user_names);
//! 
//! // Check out Hikaru's first game on Chess.com:
//! dbg!(&games[0]);
//! ```
//! 
//! ## Future plans
//! 
//! Create a stockfish wrapper so that you can analyze all your games. The game data include all the moves made in those games, so this can be fed into the engine for a variety of analyses.
//! 

use reqwest::blocking::get;
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct GameUrls {
    archives: Vec<String>
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="lowercase")]
pub enum TimeClass {
    Bullet,
    Blitz,
    Rapid,
    Daily,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="lowercase")]
pub enum Rules {
    Chess,
    Chess960,
    CrazyHouse,
    ThreeCheck,
    KingOfTheHill,
    Horde,
    BugHouse,
    OddsChess
}


#[derive(Debug, Deserialize)]
struct Game {
    #[serde(rename="url")]
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
    #[serde(rename="match")]
    team_match: Option<String>,
    white: Player,
    black: Player,
}

#[derive(Debug, Serialize)]
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
    #[serde(rename="match")]
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

impl From<(Game, &str)> for GameData {
    fn from (game_data: (Game, &str)) -> Self {
        let game = game_data.0;
        let user = game_data.1;
        let pgn: PGN = game.pgn.into();
        
        let is_white = user == game.white.username;

        let result = if is_white {game.white.result} else {game.black.result};
        let result_win_lose = result.into();
        let rating = if is_white {game.white.rating} else {game.black.rating};
        let colour = if is_white {"White"} else {"Black"};

        
        let win = 
            match result_win_lose {
                GameResultWinLose::Win => 1.0,
                GameResultWinLose::Draw => 0.5,
                GameResultWinLose::Loss => 0.0,
            };

        GameData {
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




#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
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
    #[serde(rename="50move")]
    FiftyMove,
    TimeVsInsufficient,
    KingOfTheHill,
    ThreeCheck,
    BugHousePartnerLose,
    BugHousePartnerWin,
}

#[derive(Debug, Serialize)]
pub enum GameResultWinLose {
    Win,
    Loss,
    Draw,
}

impl From<GameResult> for GameResultWinLose {
    fn from( res: GameResult) -> Self {
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
    #[serde(rename="@id")]
    id: String,
}

#[derive(Debug, Deserialize)]
struct Games {
    games: Vec<Game>
}

#[allow(non_snake_case)]
#[derive(Debug)]
struct PGN {
    ECO: String,
    ECO_url: String,
    UTC_date: String,
    UTC_time: String,
    start_date: String,
    start_time: String,

}

impl From<Option<String>> for PGN {
    fn from (pgn: Option<String>) -> Self {
        
        let mut eco = String::new();
        let mut eco_url = String::new();
        let mut utc_date = String::new();

        if let Some(pgn_data) = pgn {
            for line in pgn_data.split('\n') {
                if line.starts_with("[ECO \"") {eco = line.split(" ").nth(1).unwrap_or("").replace('"',"").replace(']',"").into();}
                if line.starts_with("[ECOUrl \"") {eco_url = line.split(" ").nth(1).unwrap_or("").replace('"',"").replace(']',"").into();}
                if line.starts_with("[UTCDate \"") {utc_date = line.split(" ").nth(1).unwrap_or("").replace('"',"").replace(']',"").into();}
            }
            Self {
                ECO: eco,
                ECO_url: eco_url,
                UTC_date: utc_date,
                UTC_time: "".to_string(),
                start_time: "".to_string(),
                start_date: "".to_string(),
            }
        }

        else {
            Self {
                ECO: "".to_string(),
                ECO_url: "".to_string(),
                UTC_date: "".to_string(),
                UTC_time: "".to_string(),
                start_time: "".to_string(),
                start_date: "".to_string(),
            }
        }
        
    }
}

fn get_game_month_urls (user: &str) -> Vec <String> {

    let url = format!("https://api.chess.com/pub/player/{}/games/archives", user);
    
    let text = get(&url)
        .expect("Didn't get a response")
        .text()
        .expect("Invalid response");
    
    let game_urls: GameUrls = serde_json::from_str(&text).expect("Serde error!");

    game_urls.archives

}

fn get_games (game_archive_urls: Vec<String>) -> Vec<Game> {

    let mut games: Vec<Game> = vec![];

    for game_month in game_archive_urls {

        let games_text = get(&game_month)
            .expect("Didn't get a response")
            .text()
            .expect("Invalid response");

        let month_games: Games = serde_json::from_str(&games_text).expect("Serde error!");
        games.extend(month_games.games)

    }

    games
}

impl GameData {
    pub fn download (users: Vec<&str>) -> Vec<GameData> {
        let mut game_data = vec![];
        for user in users {
            let urls = get_game_month_urls(&user);
            let games = get_games(urls);
            let game_data_user: Vec<GameData> = games.into_iter()
                               .map(|game| (game, user).into())
                               .collect()
                               ;
            
        game_data.extend(game_data_user);
        }
    game_data
    }
}