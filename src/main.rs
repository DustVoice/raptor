use std::sync::Arc;

use crate::raptor::Raptor;

mod data;
mod error;
mod gtfs;
mod raptor;

fn deserialize_into<T: serde::de::DeserializeOwned>(path: &str) -> Vec<T> {
    csv::Reader::from_path(path)
        .unwrap()
        .deserialize()
        .map(|parse_result| parse_result.unwrap())
        .collect()
}

fn main() {
    println!("Processing stops");
    let stops = deserialize_into::<gtfs::Stop>("timetable/stops.txt");

    println!("Processing routes");
    let routes = deserialize_into::<gtfs::Route>("timetable/routes.txt");

    println!("Processing trips");
    let trips = deserialize_into::<gtfs::Trip>("timetable/trips.txt");

    println!("Processing stop_times");
    let stop_times = deserialize_into::<gtfs::StopTime>("timetable/stop_times.txt");

    println!("Processing transfers");
    let transfers = deserialize_into::<gtfs::Transfer>("timetable/transfers.txt");

    println!("Creating gtfs_timetable");
    let gtfs_timetable = gtfs::Timetable {
        stops,
        routes,
        trips,
        stop_times,
        transfers,
    };

    println!("Creating timetable");
    let timetable = Arc::new(data::Timetable::from(gtfs_timetable));

    println!("Searching for starting stop");
    let starting_stop = timetable
        .stops
        .get("8530813")
        .expect("Starting stop should exist");

    println!("Searching for target stop");
    let target_stop = timetable
        .stops
        .get("8574471")
        .expect("Target stop should exist");

    println!("Creating Raptor instance");
    let mut raptor = Raptor::new(
        Arc::clone(&timetable),
        Arc::clone(starting_stop),
        0,
        Some(Arc::clone(target_stop)),
    );

    println!("Running Raptor");
    raptor.run();

    println!(
        "Last round at target stop: {:?}",
        raptor.rounds.last().unwrap().tau.get(target_stop).unwrap()
    )
}
