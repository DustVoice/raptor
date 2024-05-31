use std::{collections::HashMap, sync::Arc};

use crate::{data::*, timetable::*};

pub type Tau = HashMap<Arc<Stop>, Time>;

#[derive(Debug)]
pub struct Raptor {
    pub timetable: Arc<Timetable>,

    pub rounds: Vec<Round>,
    pub tau_min: Tau,
    pub marked_stops: Vec<Arc<Stop>>,
}

#[derive(Debug, Default)]
pub struct Round {
    pub tau: Tau,
}

impl Raptor {
    pub fn new(timetable: &Arc<Timetable>) -> Self {
        Self {
            timetable: Arc::clone(timetable),
            rounds: Vec::new(),
            tau_min: Tau::new(),
            marked_stops: Vec::new(),
        }
    }

    pub fn init(&mut self, starting_stop: &Arc<Stop>, starting_time: Time) {
        let mut starting_round = Round::default();

        starting_round
            .tau
            .insert(Arc::clone(starting_stop), starting_time);
        self.tau_min
            .insert(Arc::clone(starting_stop), starting_time);
        self.marked_stops.push(Arc::clone(starting_stop));

        self.rounds.push(starting_round);
    }

    pub fn round(&mut self) {
        let mut curr_round = Round::default();

        for queue_item in self.timetable.enqueue(&self.marked_stops).as_vec() {
            let mut current_trip: Option<Arc<Trip>> = None;

            for stop in queue_item.stops() {
                if let (Some(arr), Some(&tau_min)) = (
                    Trip::arr(current_trip.as_deref(), &stop),
                    self.tau_min.get(&stop),
                ) {
                    if arr < tau_min {
                        curr_round.tau.insert(Arc::clone(&stop), arr);
                        self.tau_min.insert(Arc::clone(&stop), arr);
                        self.marked_stops.push(Arc::clone(&stop))
                    }
                }

                if let Some(&tau_last) = self
                    .rounds
                    .last()
                    .expect("Raptor.rounds should contain at least a single item at this poing")
                    .tau
                    .get(&stop)
                {
                    if let Some(dep) = Trip::dep(current_trip.as_deref(), &stop) {
                        if !dep >= tau_last {
                            continue;
                        }

                        current_trip = queue_item
                            .route
                            .trips
                            .iter()
                            .filter_map(|trip| Some(Trip::dep(Some(trip), &stop)? >= &tau_last))
                    }
                }
            }
        }

        self.rounds.push(curr_round);
    }

    pub fn is_finished(&self) -> bool {
        !self.rounds.is_empty() && self.marked_stops.is_empty()
    }
}
