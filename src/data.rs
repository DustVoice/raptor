use itertools::*;
use std::{collections::HashMap, hash::Hash, sync::Arc};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Stop {
    pub id: String,
}

pub type Time = u32;

#[derive(Debug, PartialEq, Eq)]
pub struct StopTime {
    pub stop: Arc<Stop>,
    pub arrival_time: Time,
    pub departure_time: Time,
}

#[derive(Debug, PartialEq, Eq)]
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
}

#[derive(Debug)]
pub struct RawRoute {
    pub id: String,
    pub trips: Vec<Arc<Trip>>,
}

impl PartialEq for RawRoute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Eq)]
pub struct Route {
    pub parent_id: String,
    pub group_id: String,
    pub trips: Vec<Arc<Trip>>,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Trip {
    pub fn stops(&self) -> Vec<Arc<Stop>> {
        self.stop_times
            .iter()
            .map(|stop_time| Arc::clone(&stop_time.stop))
            .collect()
    }

    pub fn stop_time(&self, stop: &Stop) -> Option<Arc<StopTime>> {
        self.stop_times
            .iter()
            .find(|stop_time| stop_time.stop.as_ref() == stop)
            .cloned()
    }

    pub fn arr(trip: Option<&Trip>, stop: &Arc<Stop>) -> Option<Time> {
        Some(trip.as_ref()?.stop_time(stop)?.arrival_time)
    }

    pub fn dep(trip: Option<&Trip>, stop: &Arc<Stop>) -> Option<Time> {
        Some(trip.as_ref()?.stop_time(stop)?.departure_time)
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

impl From<RawRoute> for Vec<Route> {
    fn from(val: RawRoute) -> Self {
        val.trip_groups()
            .into_iter()
            .map(|(group_id, group)| Route {
                parent_id: val.id.to_owned(),
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

#[derive(Debug)]
pub struct QueueItem {
    pub route: Arc<Route>,
    pub hop_stop: Arc<Stop>,
}

impl QueueItem {
    pub fn precedes(&self, other: &QueueItem) -> Option<bool> {
        if self.conflicts(other) {
            let stops = self.route.stops();
            if let (Some(self_pos), Some(other_pos)) = (
                stops.iter().position(|stop| stop == &self.hop_stop),
                stops.iter().position(|stop| stop == &other.hop_stop),
            ) {
                return Some(self_pos < other_pos);
            }
        }

        None
    }

    pub fn conflicts(&self, other: &QueueItem) -> bool {
        self.route == other.route
    }

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
        value
            .0
            .into_iter()
            .map(|pair| QueueItem::from(pair))
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Queue(pub HashMap<Arc<Route>, Arc<Stop>>);

impl Queue {
    pub fn insert(&mut self, item: QueueItem) {
        self.0
            .entry(item.route)
            .and_modify(|queue_item| *queue_item = Arc::clone(&item.hop_stop))
            .or_insert(item.hop_stop);
    }

    pub fn as_vec(self) -> Vec<QueueItem> {
        Vec::<QueueItem>::from(self)
    }
}

#[derive(Debug)]
pub struct Transfer {
    pub from: Arc<Stop>,
    pub to: Arc<Stop>,
    pub time: Time,
}
