use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ProcessingError {
    #[error("No such Stop {stop_id:?} on Trip {trip_id:?}")]
    NoStopOnTrip { stop_id: String, trip_id: String },

    #[error("No such Stop {stop_id:?} on Route {route_id:?}")]
    NoStopOnRoute { stop_id: String, route_id: String },

    #[error("The two queue items are unrelated as they don't share the same route")]
    UnrelatedQueueItems,
}
