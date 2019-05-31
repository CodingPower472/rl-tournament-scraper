
use std::error::Error;
use std::slice::Iter;

use crate::structures::{Game, Team, Match, Tournament};
use select::document::{Document, Find};
use select::predicate::{Class, Name, Predicate};

use chrono::NaiveDate;

const BASE_URL : &'static str = "https://liquipedia.net";

fn get_raw_html(url: &str) -> Result<Document, Box<dyn Error>> {
    let res = reqwest::get(url)?;
    Ok(Document::from_read(res)?)
}

fn get_redirected_url(url : &str) -> String {
    match reqwest::get(&(BASE_URL.to_owned() + url)).ok() {
        Some(res) => res.url().as_str().to_owned(),
        None => url.to_owned(),
    }
}

fn find_team(id : &str, name : &Option<String>, mut teams : Iter<Team>) -> Option<Team> {
    teams.clone().find(|team| team.link.to_lowercase() == id.to_lowercase() || Some(team.name.to_lowercase()) == name.clone().and_then(|name| Some(name.to_lowercase()))).map(|t| t.clone())
}

fn get_matches_in_tournament(html : &Document, teams : &Vec<Team>) -> Vec<Match> {
    let popups = html.find(Class("bracket-popup"));
    let matches = Vec::new();
    for popup in popups {
        let header = popup.find(Class("bracket-popup-header")).next();
        match header {
            Some(header) => {
                let left = header.find(Class("bracket-popup-header-left")).next();
                let right = header.find(Class("bracket-popup-header-right")).next();
                fn get_name(elem : &Option<select::node::Node<'_>>) -> Option<String> {
                    Some((*elem)?.find(Name("span")).next()?.attr("data-highlightingclass")?.to_owned())
                }
                fn get_id(elem : &Option<select::node::Node<'_>>) -> Option<String> {
                    Some(get_redirected_url((*elem)?.find(Name("span")).next()?.find(Class("team-template-text")).next()?.find(Name("a")).next()?.attr("href")?))
                }
                fn get_time(elem : &Option<select::node::Node<'_>>) -> Option<NaiveDate> {
                    Some(NaiveDate::parse_from_str(&((*elem)?).find(Class("bracket-popup-body-time")).next()?.find(Class("timer-object").and(Name("span"))).next()?.text().split("-").next()?.trim(), "%B %e, %Y").ok()?)
                }
                let body = popup.find(Class("bracket-popup-body")).next();
                let left_name = get_name(&left);
                let right_name = get_name(&right);
                let left_team = get_id(&left).and_then(|id| find_team(&id, &left_name, teams.iter()));
                let right_team = get_id(&right).and_then(|id| find_team(&id, &right_name, teams.iter()));
                let time = get_time(&body);
                match (left_name, right_name, left_team, right_team, time) {
                    (Some(left_name), Some(right_name), Some(left), Some(right), Some(time)) => {
                    },
                    _ => {
                        if NaiveDate::from_ymd(2019, 5, 30) < time.unwrap() {
                            println!("Match hasn't happened yet");
                        } else {
                            println!("Couldn't find names");
                            println!("{:?}", time);
                        }
                        //println!("{:?}", teams);
                    }
                }
            },
            None => continue,
        }

    }
    matches
}

fn get_teams_in_tournament(html : &Document) -> Result<Vec<Team>, Box<dyn Error>> {
    let teamcards = html.find(Class("teamcard"));
    Ok(teamcards.filter_map(|card| {
        let name_url_elem = card.find(Name("center")).next()?.find(Name("b")).next()?.find(Name("a")).next()?;
        let name = name_url_elem.text();
        let url = get_redirected_url(name_url_elem.attr("href")?);
        let members = card.find(Class("teamcard-inner")).next()?.find(Class("table")).next()?.find(Name("tr")).filter_map(|tr| {
            let num = tr.find(Name("th")).next()?.text().parse::<u8>().ok()?; // if this doesn't work, it's not a number so it's a sub or coach so disregard it anyways
            if num > 3 {
                return None;
            }
            let player_name = tr.find(Name("a")).filter(|a| !a.children().any(|child| child.name() == Some("img"))).map(|a| a.text()).next();
            player_name.to_owned()
        }).collect::<Vec<String>>();
        Some(Team {
            name: name,
            link: url,
            players: members,
        })
    }).collect::<Vec<Team>>())
}

pub fn get_tournament(url : &str, premier : bool) -> Result<Option<Tournament>, Box<dyn Error>> {
    println!("get tournament {}", url);
    let html = get_raw_html(url)?;
    let teams = get_teams_in_tournament(&html)?;
    let matches = get_matches_in_tournament(&html, &teams);
    Ok(None)
}

fn get_tournaments_in_event(url : &str, premier : bool, visited_navs : Vec<String> /* strings represent arrays of id's of navs*/) -> Option<Tournament> {
    let html = get_raw_html(url).ok()?;
    
}

pub fn get_all_tournaments(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(get_raw_html(url)?
        .find(Class("divTable"))
        .flat_map(|tourney_wrapper| {
            tourney_wrapper
                .find(Class("divRow"))
                .filter_map(|row| {
                    row.find(
                        Class("divCell")
                            .and(Class("Tournament"))
                            .and(Class("Header")),
                    )
                    .next()
                })
                .filter_map(|cell| cell.find(Name("b")).next())
                .filter_map(|b| b.find(Name("a")).next())
                .filter_map(|a| Some(a.attr("href")?.to_owned()))
                .map(|link| BASE_URL.to_owned() + &link)
        }).collect::<Vec<String>>())
}
