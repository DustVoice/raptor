use std::sync::Arc;

use crate::data::*;

#[derive(Debug)]
pub struct Timetable {
    pub stops: Vec<Arc<Stop>>,
    pub routes: Vec<Arc<Route>>,
    pub trips: Vec<Arc<Trip>>,
    pub transfers: Vec<Arc<Transfer>>,
}
