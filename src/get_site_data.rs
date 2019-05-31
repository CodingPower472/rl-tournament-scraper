
use std::error::Error;
use std::slice::Iter;
use std::collections::HashMap;

use crate::structures::{Game, Event, Team, Match, Tournament};
use select::document::{Document};
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

fn get_matches_in_tournament(html : Document, teams : &Vec<Team>) -> Vec<Match> {
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
                fn find_match_bodies(elem : Option<select::node::Node<'static>>) -> Option<Vec<(u8, &'static str)>> {
                    Some(elem?.find(Name("div")).flat_map(|container| container.find(Class("div"))).filter_map(|inner| Some((inner.text().parse::<u8>().ok()?, inner.attr("style")?))).collect::<Vec<(u8, &'static str)>>())
                }
                let body = popup.find(Class("bracket-popup-body")).next();
                let left_name = get_name(&left);
                let right_name = get_name(&right);
                let left_team = get_id(&left).and_then(|id| find_team(&id, &left_name, teams.iter()));
                let right_team = get_id(&right).and_then(|id| find_team(&id, &right_name, teams.iter()));
                let time = get_time(&body);
                match (left_name, right_name, left_team, right_team, time) {
                    (Some(left_name), Some(right_name), Some(left), Some(right), Some(time)) => {
                        let mut games = Vec::new();
                        let mut pending : Option<(u8, bool)> = None; // true for bool represents left
                        let match_bodies = find_match_bodies(body);
                        if let Some(match_bodies) = match_bodies {
                            for (score, style) in match_bodies {
                                let is_left = style.contains("left");
                                match Some(is_left) != pending.map(|p| p.1) {
                                    true => {
                                        pending = None;
                                        let pending_score = pending.unwrap().0; // we know this is Some due to earlier match statement
                                        let mut scores = HashMap::new();
                                        if is_left {
                                            scores.insert(left.clone(), score);
                                            scores.insert(right.clone(), pending_score);
                                        } else {
                                            scores.insert(left.clone(), pending_score);
                                            scores.insert(right.clone(), score);
                                        }
                                        games.push(Game {
                                            scores: scores
                                        });
                                    },
                                    false => {
                                        pending = Some((score, is_left));
                                    },
                                }
                            }
                        }
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

fn get_teams_in_tournament(html : Document) -> Result<Vec<Team>, Box<dyn Error>> {
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

pub fn get_tournament(html : Document, premier : bool, name : String) -> Result<Tournament, Box<dyn Error>> {
    let teams = get_teams_in_tournament(html.clone())?;
    let matches = get_matches_in_tournament(html.clone(), &teams);
    let lan = html.find(Class("infobox-cell-2")).next().map(|e| e.text()) == Some("Offline".to_string());
    Ok(Tournament {
        teams: teams,
        matches: matches,
        lan: lan,
        name: name,
    })
}

pub fn get_tournament_from_url(url : &str, premier : bool, name : String) -> Result<Tournament, Box<dyn Error>> {
    println!("get tournament {}", url);
    get_tournament(get_raw_html(url)?, premier, name)
}

fn get_tournaments_in_event(html : Document, premier : bool, visited_navs : Vec<String> /* strings represent arrays of id's of navs*/) -> Vec<Tournament> {
    let nav = html.find(Class("tabs-static")).filter(|nav| nav.attr("id").is_some()).find(|nav| visited_navs.contains(&nav.attr("id").unwrap().to_owned()));
    let mut new_vn = visited_navs;
    match nav {
        Some(nav) => {
            new_vn.push(nav.attr("id").unwrap().to_owned());
            let tabs = nav.find(Name("a"));
            let tabs_urls = tabs.filter_map(|a| Some((BASE_URL.to_owned() + a.attr("href")?, a.text())));
            let tab_current = nav.find(Class("selflink")).next();

            let mut new_vec = tabs_urls.map(|(tab_url, tab_title)| get_tournament_from_url(&tab_url, premier, tab_title).unwrap()).collect::<Vec<Tournament>>();
            if let Some(tab_current) = tab_current {
                match get_tournament(html.clone(), premier, tab_current.text()) {
                    Ok(tournament) => new_vec.push(tournament),
                    _ => (),
                };
            }
            let inner = get_tournaments_in_event(html.clone(), premier, new_vn);
            new_vec.extend(inner);
            new_vec
        },
        None => Vec::new(),
    }
}

fn get_event(url : &str, premier : bool) -> Option<Event> {
    let html = get_raw_html(url).ok()?;
    let name = html.find(Class("firstHeading")).next()?.find(Name("span")).next()?.text();
    let tournaments = get_tournaments_in_event(html, premier, Vec::new());
    Some(Event {
        name: name,
        premier: premier,
        tournaments: tournaments
    })
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
