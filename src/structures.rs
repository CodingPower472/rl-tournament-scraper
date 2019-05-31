
use chrono::{NaiveDate};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Team {
    pub name : String,
    pub link : String,
    pub players : Vec<String>,
}

#[derive(Clone)]
pub struct Game {
    pub scores : HashMap<Team, u8>,
}

#[derive(Clone)]
pub struct Match {
    pub date : NaiveDate,
    pub teams : Vec<Team>,
    pub games : Vec<Game>,
}

#[derive(Clone)]
pub struct Tournament {
    pub name : String,
    pub premier : bool,
    pub lan : bool,
    pub teams : Vec<Team>,
    pub matches : Vec<Match>,
}
