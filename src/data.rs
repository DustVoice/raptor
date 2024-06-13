use itertools::*;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::error::ProcessingError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stop {
    pub id: String,
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

pub type Time = u32;

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
    pub trips: Vec<Arc<Trip>>,
}

impl PartialEq for RawRoute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Eq)]
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
