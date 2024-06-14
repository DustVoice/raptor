use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Route {
    pub route_id: String,
    pub route_short_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Trip {
    pub route_id: String,
    pub trip_id: String,
    pub trip_short_name: String,
}

#[derive(Deserialize, Debug)]
pub struct StopTime {
    pub trip_id: String,
    pub arrival_time: String,
    pub departure_time: String,
    pub stop_id: String,
    pub stop_sequence: u32,
}

#[derive(Deserialize, Debug)]
pub struct Transfer {
    pub from_stop_id: String,
    pub to_stop_id: String,
    pub min_transfer_time: u32,
}

#[derive(Debug)]
pub struct Timetable {
    pub stops: Vec<Stop>,
    pub routes: Vec<Route>,
    pub trips: Vec<Trip>,
    pub stop_times: Vec<StopTime>,
    pub transfers: Vec<Transfer>,
}
