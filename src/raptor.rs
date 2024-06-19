use std::{cmp, collections::HashMap, sync::Arc};

use crate::data::*;

pub type Tau = HashMap<Arc<Stop>, Time>;

#[derive(Debug)]
pub struct Raptor {
    pub timetable: Arc<Timetable>,
    pub stops_for_routes: HashMap<Arc<Route>, Vec<Arc<Stop>>>,

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
            timetable: Arc::clone(&timetable),
            stops_for_routes: timetable.stops_for_routes(),
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
            println!("Running Raptor round {:?}", self.rounds.len());
            self.round();
        }
    }

    pub fn round(&mut self) {
        let mut curr_round = Round::default();

        let queue = Queue::enqueue(&self.marked_stops, &self.stops_for_routes).as_vec();
        self.marked_stops.clear();

        for queue_item in queue {
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
                        self.marked_stops.push(Arc::clone(&stop));

                        let faulty_stop = self.timetable.stops.get("8503059").unwrap();
                        if stop.id == faulty_stop.id {
                            println!(
                                "Culprit stop {:?} found in round: {}",
                                faulty_stop,
                                self.rounds.len()
                            );
                            println!(
                                "Stop {} found in curr_round.tau: {}",
                                faulty_stop.id,
                                curr_round.tau.contains_key(faulty_stop)
                            );
                            println!(
                                "Stop {} found in tau_min: {}",
                                faulty_stop.id,
                                self.tau_min.contains_key(faulty_stop)
                            );
                            println!(
                                "Stop {} found in marked_stops: {}",
                                faulty_stop.id,
                                self.marked_stops.contains(faulty_stop)
                            );
                        }
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
            let transfer_time = curr_round.tau.get(&transfer.from).unwrap_or_else(||
                panic!("Transfer.from must be present in Raptor.tau, as it is in Raptor.marked_stops.\n-> Current Transfer: {:?}\n-> Marked Stops: {:?}\n-> Current Round Tau: {:?}",
                    transfer,
                    self.marked_stops.iter().map(|stop| stop.id.to_owned()).collect::<Vec<ID>>(),
                    curr_round.tau.keys().map(|tau_stops| tau_stops.id.to_owned()).collect::<Vec<ID>>()
                ),
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
