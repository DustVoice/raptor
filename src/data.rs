use itertools::*;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::{error::ProcessingError, gtfs};

pub type Time = u32;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stop {
    pub id: String,
    pub name: String,
}

impl Stop {
    pub fn precedes(&self, other: &Stop, route: &Route) -> Result<bool, ProcessingError> {
        let stops = route.stops();
        Ok(stops.iter().position(|stop| stop.as_ref() == self).ok_or(
            ProcessingError::NoStopOnRoute {
                stop_id: self.id.to_owned(),
                route_id: route.id(),
            },
        )? < stops.iter().position(|stop| stop.as_ref() == other).ok_or(
            ProcessingError::NoStopOnRoute {
                stop_id: other.id.to_owned(),
                route_id: route.id(),
            },
        )?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StopTime {
    pub stop: Arc<Stop>,
    pub arrival_time: Time,
    pub departure_time: Time,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trip {
    id: String,
    stop_times: Vec<Arc<StopTime>>,
}

impl Trip {
    pub fn group_id(&self) -> String {
        self.stop_times
            .iter()
            .fold(String::default(), |acc, stop_time| {
                format!("{}_{}", acc, stop_time.stop.id)
            })
    }

    pub fn stops(&self) -> Vec<Arc<Stop>> {
        self.stop_times
            .iter()
            .map(|stop_time| Arc::clone(&stop_time.stop))
            .collect()
    }

    pub fn stop_time(&self, stop: &Stop) -> Result<Arc<StopTime>, ProcessingError> {
        self.stop_times
            .iter()
            .find(|stop_time| stop_time.stop.as_ref() == stop)
            .cloned()
            .ok_or(ProcessingError::NoStopOnTrip {
                stop_id: self.id.to_owned(),
                trip_id: stop.id.to_owned(),
            })
    }

    pub fn arr(&self, stop: &Stop) -> Result<Time, ProcessingError> {
        Ok(self.stop_time(stop)?.arrival_time)
    }

    pub fn dep(&self, stop: &Stop) -> Result<Time, ProcessingError> {
        Ok(self.stop_time(stop)?.departure_time)
    }
}

#[derive(Clone, Debug)]
pub struct RawRoute {
    pub id: String,
    pub name: String,
    pub trips: Vec<Arc<Trip>>,
}

impl PartialEq for RawRoute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl RawRoute {
    pub fn trip_groups(&self) -> HashMap<String, Vec<Arc<Trip>>> {
        self.trips
            .iter()
            .map(|trip| (trip.group_id(), Arc::clone(trip)))
            .into_group_map()
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Route {
    pub parent_id: String,
    pub parent_name: String,
    pub group_id: String,
    pub trips: Vec<Arc<Trip>>,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl From<RawRoute> for Vec<Route> {
    fn from(val: RawRoute) -> Self {
        val.trip_groups()
            .into_iter()
            .map(|(group_id, group)| Route {
                parent_id: val.id.to_owned(),
                parent_name: val.name.to_owned(),
                group_id: group_id.to_owned(),
                trips: group,
            })
            .collect()
    }
}

impl Route {
    pub fn id(&self) -> String {
        format!("{}_{}", self.parent_id, self.group_id)
    }

    pub fn stops(&self) -> Vec<Arc<Stop>> {
        match self.trips.first() {
            Some(first_trip) => first_trip
                .stop_times
                .iter()
                .map(|stop_time| Arc::clone(&stop_time.stop))
                .collect(),
            None => Vec::default(),
        }
    }
}

impl Hash for Route {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

#[derive(Clone, Debug)]
pub struct QueueItem {
    pub route: Arc<Route>,
    pub hop_stop: Arc<Stop>,
}

impl QueueItem {
    pub fn stops(&self) -> Vec<Arc<Stop>> {
        self.route
            .stops()
            .iter()
            .skip_while(|&stop| stop != &self.hop_stop)
            .cloned()
            .collect()
    }
}

impl From<(Arc<Route>, Arc<Stop>)> for QueueItem {
    fn from(value: (Arc<Route>, Arc<Stop>)) -> Self {
        Self {
            route: value.0,
            hop_stop: value.1,
        }
    }
}

impl From<Queue> for Vec<QueueItem> {
    fn from(value: Queue) -> Self {
        value.0.into_iter().map(QueueItem::from).collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Queue(pub HashMap<Arc<Route>, Arc<Stop>>);

impl Queue {
    pub fn insert(&mut self, item: QueueItem) {
        self.0
            .entry(Arc::clone(&item.route))
            .and_modify(|queue_entry| {
                if item
                    .hop_stop
                    .precedes(queue_entry.as_ref(), &item.route)
                    .unwrap()
                {
                    *queue_entry = Arc::clone(&item.hop_stop)
                }
            })
            .or_insert(item.hop_stop);
    }

    pub fn as_vec(&self) -> Vec<QueueItem> {
        Vec::<QueueItem>::from(self.clone())
    }

    pub fn enqueue(marked_stops: &[Arc<Stop>], routes: &[Arc<Route>]) -> Self {
        let mut queue = Self::default();

        for marked_stop in marked_stops {
            for route in routes
                .iter()
                .filter(|route| route.stops().contains(marked_stop))
            {
                queue.insert(QueueItem {
                    route: Arc::clone(route),
                    hop_stop: Arc::clone(marked_stop),
                })
            }
        }

        queue
    }
}

#[derive(Debug)]
pub struct Transfer {
    pub from: Arc<Stop>,
    pub to: Arc<Stop>,
    pub time: Time,
}

#[derive(Debug)]
pub struct Timetable {
    pub stops: Vec<Arc<Stop>>,
    pub routes: Vec<Arc<Route>>,
    pub trips: Vec<Arc<Trip>>,
    pub transfers: Vec<Arc<Transfer>>,
}

impl From<gtfs::Timetable> for Timetable {
    fn from(value: gtfs::Timetable) -> Self {
        println!("Converting stops");
        let stops: Vec<Arc<Stop>> = value
            .stops
            .into_iter()
            .map(|stop| {
                Arc::new(Stop {
                    id: stop.stop_id,
                    name: stop.stop_name,
                })
            })
            .collect();

        let stop_by_id = |id: String, stops: &Vec<Arc<Stop>>| -> Arc<Stop> {
            Arc::clone(
                stops
                    .iter()
                    .find(|stop| stop.id == id)
                    .expect("Stop not present in list of stops"),
            )
        };

        let time_to_seconds = |time: String| -> u32 {
            let components: Vec<String> = time.split(':').map(String::from).collect();
            assert!(components.len() == 3);
            (components[0].parse::<u32>().unwrap() * 60 * 60)
                + (components[1].parse::<u32>().unwrap() * 60)
                + components[2].parse::<u32>().unwrap()
        };

        println!("Calculating StopTimes-Hashmap");
        let stop_times_intermed = value
            .stop_times
            .iter()
            .map(|stop_time| {
                (
                    stop_time.trip_id.to_owned(),
                    Arc::new(StopTime {
                        stop: stop_by_id(stop_time.stop_id.to_owned(), &stops),
                        arrival_time: time_to_seconds(stop_time.arrival_time.to_owned()),
                        departure_time: time_to_seconds(stop_time.departure_time.to_owned()),
                    }),
                )
            })
            .into_group_map();

        println!("Calculating Trips-Hashmap");
        let trips_intermed =
            value
                .trips
                .iter()
                .map(|trip| {
                    (
                    trip.route_id.to_owned(),
                    Arc::new(Trip {
                        id: trip.trip_id.to_owned(),
                        stop_times: stop_times_intermed.get(&trip.trip_id)

                    .expect("Key corresponding to Trip should be present in StopTimes-Hashmap")
                    .to_owned(),
                    }),
                )
                })
                .into_group_map();

        println!("Converting RawRoutes");
        let raw_routes: Vec<RawRoute> = value
            .routes
            .into_iter()
            .map(|route| RawRoute {
                id: route.route_id.to_owned(),
                name: route.route_short_name,
                trips: trips_intermed
                    .get(&route.route_id)
                    .expect("Key corresponding to Route should be present in Trips-Hashmap")
                    .to_owned(),
            })
            .collect();

        println!("Extracting trips");
        let trips = raw_routes
            .iter()
            .flat_map(|raw_route| raw_route.trips.clone())
            .collect();

        println!("Converting Routes");
        let routes = raw_routes
            .into_iter()
            .flat_map(|raw_route| {
                Vec::<Route>::from(raw_route)
                    .into_iter()
                    .map(Arc::new)
                    .collect::<Vec<Arc<Route>>>()
            })
            .collect();

        println!("Converting Transfers");
        let transfers = value
            .transfers
            .into_iter()
            .map(|transfer| {
                Arc::new(Transfer {
                    from: stop_by_id(transfer.from_stop_id, &stops),
                    to: stop_by_id(transfer.to_stop_id, &stops),
                    time: transfer.min_transfer_time,
                })
            })
            .collect();

        Self {
            stops,
            routes,
            trips,
            transfers,
        }
    }
}
