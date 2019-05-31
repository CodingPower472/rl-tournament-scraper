
use chrono::{NaiveDate};
use std::collections::HashMap;

#[derive(Clone, Debug, Hash)]
pub struct Team {
    pub name : String,
    pub link : String,
    pub players : Vec<String>,
}

impl PartialEq for Team {
    fn eq(&self, rhs : &Self) -> bool {
        self.link == rhs.link
    }
}

impl Eq for Team {}

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
    pub lan : bool,
    pub teams : Vec<Team>,
    pub matches : Vec<Match>,
}

pub struct Event {
    pub name : String,
    pub premier : bool,
    pub tournaments : Vec<Tournament>,
}
