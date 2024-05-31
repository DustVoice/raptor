use std::sync::Arc;

use crate::data::*;

#[derive(Debug)]
pub struct Timetable {
    pub stops: Vec<Arc<Stop>>,
    pub routes: Vec<Arc<Route>>,
    pub trips: Vec<Arc<Trip>>,
    pub transfers: Vec<Arc<Transfer>>,
}

impl Timetable {
    pub fn enqueue(&self, marked_stops: &Vec<Arc<Stop>>) -> Queue {
        let mut queue = Queue::default();

        for marked_stop in marked_stops {
            for route in self
                .routes
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
