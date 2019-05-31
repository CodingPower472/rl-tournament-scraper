mod get_site_data;
mod structures;

use get_site_data::{get_all_tournaments, get_tournament_from_url};

const MAJOR_TOURNAMENTS_LINK : &'static str = "https://liquipedia.net/rocketleague/Major_Tournaments";
const PREMIER_TOURNAMENTS_LINK : &'static str = "https://liquipedia.net/rocketleague/Premier_Tournaments";

fn main() {
    let major_urls = get_all_tournaments(MAJOR_TOURNAMENTS_LINK).expect("Error extracting major tournaments");
    let premier_urls = get_all_tournaments(PREMIER_TOURNAMENTS_LINK).expect("Error extracting premier tournaments");
    /*let major_tourneys = major_urls.iter().for_each(|url| {
        get_tournament_from_url(url, false);
    });
    let premier_tourneys = premier_urls.iter().map(|url| get_tournament_from_url(url, true));*/
}
