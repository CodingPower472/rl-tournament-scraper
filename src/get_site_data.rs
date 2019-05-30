use std::error::Error;

use crate::structures::{Game, Team, Match, Tournament};
use select::document::{Document, Find};
use select::predicate::{Class, Name, Predicate};

use chrono::NaiveDate;

const BASE_URL : &'static str = "https://liquipedia.net";

fn get_raw_html(url: &str) -> Result<Document, Box<dyn Error>> {
    let res = reqwest::get(url)?;
    Ok(Document::from_read(res)?)
}

fn get_matches_in_tournament(html : Document) -> Vec<Match> {
    let popups = html.find(Class("bracket-popup"));
    let matches = Vec::new();
    for popup in popups {
        let header = popup.find(Class("bracket-popup-header")).next();
        match header {
            Some(header) => {
                let left = header.find(Class("bracket-popup-header-left")).next();
                let right = header.find(Class("bracket-popup-header-right")).next();
                fn get_name(elem : Option<select::node::Node<'_>>) -> Option<String> {
                    Some(elem?.find(Name("span")).next()?.attr("data-highlightingclass")?.to_owned())
                }
                fn get_id(elem : Option<select::node::Node<'_>>) -> Option<String> {
                    Some(elem?.find(Name("span")).next()?.find(Class("team-template-text")).next()?.find(Name("a")).next()?.text())
                }
                fn get_time(elem : Option<select::node::Node<'_>>) -> Option<NaiveDate> {
                    Some(NaiveDate::parse_from_str(&elem?.find(Class("bracket-popup-body-time")).next()?.find(Class("timer-object").and(Name("span"))).next()?.text().split("-").next()?.trim(), "%B %e, %Y").ok()?)
                }
                let body = popup.find(Class("bracket-popup-body")).next();
                match (get_name(left), get_name(right), get_id(left), get_id(right), get_time(body)) {
                    (Some(left_name), Some(right_name), Some(left_id), Some(right_id), Some(time)) => {
                        println!("{} vs {} ({} vs {}) at {}", left_name, right_name, left_id, right_id, time);
                    },
                    _ => {
                        println!("Couldn't find names");
                    }
                }
            },
            None => continue,
        }

    }
    matches
}

fn get_teams_in_tournament(html : Document) -> Result<Vec<Team>, Box<dyn Error>> {
    let teamcards = html.find(Class("teamcard"));
    teamcards.filter_map(|card| {
        let name_url_elem = card.find(Name("center")).next()?.find(Name("b")).next()?.find(Name("a")).next()?;
        let name = name_url_elem.text();
        let url = name_url_elem.attr("href")?;
        let members = card.find(Class("teamcard-inner")).next()?.find(Class("table")).next()?.find(Name("tr")).filter_map(|tr| {
            let num = tr.find(Name("th")).next()?.text().parse::<u8>().ok()?; // if this doesn't work, it's not a number so it's a sub or coach so disregard it anyways
            if num < 0 || num > 3 {
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
    };
}

pub fn get_tournament(url : &str, premier : bool) -> Result<Tournament, Box<dyn Error>> {
    let html = get_raw_html(url)?;
    let matches = get_matches_in_tournament(html);
    unimplemented!("Get tournament not yet implemented");
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
