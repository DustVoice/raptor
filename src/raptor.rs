use std::{cmp, collections::HashMap, sync::Arc};

use crate::data::*;

pub type Tau = HashMap<Arc<Stop>, Time>;

#[derive(Debug)]
pub struct Raptor {
    pub timetable: Arc<Timetable>,

    pub rounds: Vec<Round>,
    pub tau_min: Tau,
    pub marked_stops: Vec<Arc<Stop>>,

    pub target: Option<Arc<Stop>>,
}

#[derive(Debug, Default)]
pub struct Round {
    pub tau: Tau,
}

impl Raptor {
    pub fn new(
        timetable: Arc<Timetable>,
        starting_stop: Arc<Stop>,
        starting_time: Time,
        target: Option<Arc<Stop>>,
    ) -> Self {
        Self {
            timetable,
            rounds: vec![Round {
                tau: HashMap::from([(starting_stop.clone(), starting_time)]),
            }],
            tau_min: HashMap::from([(starting_stop.clone(), starting_time)]),

            marked_stops: vec![starting_stop],
            target,
        }
    }

    pub fn run(&mut self) {
        while !self.is_finished() {
            self.round();
        }
    }

    pub fn round(&mut self) {
        let mut curr_round = Round::default();

        for queue_item in Queue::enqueue(&self.marked_stops, &self.timetable.routes).as_vec() {
            let mut current_trip: Option<Arc<Trip>> = None;

            for stop in queue_item.stops() {
                if let Some(trip) = &current_trip {
                    let arr = trip.arr(&stop).unwrap();

                    if match (
                        self.tau_min.get(&stop),
                        self.target
                            .as_ref()
                            .and_then(|target| self.tau_min.get(target)),
                    ) {
                        (Some(&tau_pi), Some(&tau_pt)) => arr < cmp::min(tau_pi, tau_pt),
                        (Some(&tau_pi), None) => arr < tau_pi,
                        (None, Some(&tau_pt)) => arr < tau_pt,
                        _ => true,
                    } {
                        curr_round.tau.insert(Arc::clone(&stop), arr);
                        self.tau_min.insert(Arc::clone(&stop), arr);
                        self.marked_stops.push(Arc::clone(&stop))
                    }
                }

                if let Some(&tau_prev) = self
                    .rounds
                    .last()
                    .expect("Raptor.rounds should contain at least a single item at this point")
                    .tau
                    .get(&stop)
                {
                    if let Some(trip) = &current_trip {
                        if trip.dep(&stop).unwrap() < tau_prev {
                            continue;
                        };
                    }

                    current_trip = queue_item
                        .route
                        .trips
                        .iter()
                        .filter_map(|trip| {
                            let dep = trip.dep(&stop).ok()?;
                            (dep >= tau_prev).then_some((dep, trip))
                        })
                        .min_by(|x, y| x.0.cmp(&y.0))
                        .map(|(_, trip)| Arc::clone(trip));
                }
            }
        }

        for transfer in self
            .timetable
            .transfers
            .iter()
            .filter(|t| self.marked_stops.contains(&t.from))
        {
            let transfer_time = curr_round.tau.get(&transfer.from).expect(
                "Transfer.from must be present in Raptor.tau, as it is in Raptor.marked_stops",
            ) + transfer.time;
            curr_round.tau.insert(
                Arc::clone(&transfer.to),
                if let Some(&tau) = curr_round.tau.get(&transfer.to) {
                    cmp::min(tau, transfer_time)
                } else {
                    transfer_time
                },
            );
        }

        self.rounds.push(curr_round);
    }

    pub fn is_finished(&self) -> bool {
        !self.rounds.is_empty() && self.marked_stops.is_empty()
    }
}
