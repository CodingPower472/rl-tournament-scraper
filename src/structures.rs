
use chrono::{NaiveDate};
use std::collections::HashMap;

pub struct Team {
    name : String,
    link : String,
    players : Vec<String>,
}

pub struct Game {
    scores : HashMap<Team, u8>,
}

pub struct Match {
    date : NaiveDate,
    teams : Vec<Team>,
    games : Vec<Game>,
}

pub struct Tournament {
    name : String,
    premier : bool,
    lan : bool,
    teams : Vec<Team>,
}
