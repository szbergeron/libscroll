use std::collections::VecDeque;
type Timestamp = u64;
type TimeDeltaMicros = u64;

type Velocity = f64;
type Position = f64;

const MILLIS_PER_FRAME_DEFAULT: u64 = 16;

#[derive(Clone, Copy)]
enum Phase {
    Inactive,
    Interpolating,
    Released,
}

#[derive(Clone, Copy)]
struct Event {
    timestamp: Timestamp, // microseconds since interpolator init
    //delta: f64, // distance represented by this event
    value: f64, // the current absolute "position" of the event
    //bezier_forward: Option<bezier::Curve<geo::Coord2>>,
}

#[derive(Clone, Copy)]
struct Sample {
    timestamp: Timestamp,
    velocity: Velocity,
    position: Position,
}


pub struct Interpolator {
    redistributable: bool,
    // need record of samplings, matched with timestamps or
    // marked redistributable according to event/frame ratio
    //events: RangedMap<Timestamp, Event>,
    //samples: Vec<Timestamp>,
    
    events: VecDeque<Event>,
    samples: VecDeque<Sample>,
    pan_start_time: Timestamp,
    current_phase: Phase,

    // upper and lower bounds of the track, used for calculating where bouncing happens
    track_bound_upper: f64,
    track_bound_lower: f64,
    track_initial_pos: f64,
    //events_y: RangedMap<Timestamp, Event>,

}

impl Interpolator {
    pub fn new(redistributable: bool, track_bounds: (f64, f64), initial_position: f64) -> Interpolator {
        Interpolator {
            redistributable,
            events: VecDeque::with_capacity(5),
            samples: VecDeque::new(),
            pan_start_time: 0,
            current_phase: Phase::Inactive,
            track_bound_lower: track_bounds.0,
            track_bound_upper: track_bounds.1,
            track_initial_pos: initial_position,
        }
    }

    pub fn sample(&mut self, timestamp: Timestamp) -> Position {
        let velocity = self.sample_velocity(timestamp);

        let last_sample = self.samples.back().map(|&evt| evt).unwrap_or(Sample { timestamp, velocity, position: 0.0 });

        //let velocity = 0.0;
        let delta = (timestamp - last_sample.timestamp) as f64 * velocity;

        let position = last_sample.position + delta;

        // need to integrate velocity since last sample
        //let position = 0.0;

        self.samples.push_back(Sample { timestamp, velocity, position });

        position
    }

    pub fn signal_fling(&mut self, timestamp: Timestamp) {
        self.current_phase = Phase::Released;
        self.flush(timestamp);
    }

    pub fn signal_interrupt(&mut self, _timestamp: Timestamp) {
        self.current_phase = Phase::Interpolating;
    }

    pub fn signal_pan(&mut self, timestamp: Timestamp, delta: f64) {
        self.current_phase = Phase::Interpolating;

        let previous_val = self.events.back().map(|evt| evt.value).unwrap_or(self.track_initial_pos);

        let current_val = previous_val + delta;

        self.events.push_back(Event { value: current_val, timestamp });
    }

    pub fn animating(&self) -> bool {
        match self.current_phase {
            Phase::Inactive => false,
            _ => true,
        }
    }

    pub fn set_geometry(&mut self, min: f64, max: f64) {
        self.track_bound_upper = max;
        self.track_bound_lower = min;
    }
}
// Private impl
impl Interpolator {
    /// Empties the sample and event lists, used for interrupt/fling
    /// when a group of samples is logically over (a single "gesture")
    fn flush(&mut self, time: Timestamp) {
        //self.last_interpolated_velocity = self.interpolate(time);
        self.events.clear();
        //self.samples.clear(); need samples to continue animating
    }

    fn interpolate(&self, time: Timestamp) -> Velocity {
        match self.events.back() {
            None => 0.0, // no events yet, can't know if any action has started
            Some(latest) => {
                match self.events.get(1) {
                    None => latest.value * (MILLIS_PER_FRAME_DEFAULT as f64),
                    Some(second_latest) => {
                        // do Hermite interpolation later, for now just do linear (only need 2
                        // points to do properly)
                        Self::slope_of(*second_latest, *latest)
                        //match self.events.get(2)
                    }
                }
            }
        }
    }

    fn handle_overscroll(&self, velocity: Velocity) -> Velocity {
        velocity
    }

    fn accelerate(&self, velocity: Velocity) -> Velocity {
        velocity.powf(1.4)
    }

    fn decay(&self, timedelta: TimeDeltaMicros, velocity: Velocity) -> Velocity {
        velocity
    }

    fn bounce(&self, timedelta: TimeDeltaMicros, velocity: Velocity) -> Velocity {
        velocity
    }

    fn sample_velocity(&mut self, time: Timestamp) -> Velocity {
        match self.current_phase {
            Interpolating => self.accelerate(
                self.handle_overscroll(
                    self.interpolate(time))),

            Released => {
                let result = self.bounce(
                    time - self.samples.back().map(|evt| evt.timestamp).unwrap_or(time),
                    self.decay(
                        time - self.samples.back().map(|evt| evt.timestamp).unwrap_or(time),
                        self.samples.back().map(|evt| evt.velocity).unwrap_or(0.0),
                    ));

                if Self::rounds_to_zero(result) {
                    //self.current_phase = Inactive;
                    self.set_inactive();
                }

                result
            }

            Inactive => 0.0
        }
    }

    fn set_inactive(&mut self) {
        self.current_phase = Phase::Inactive;
        self.samples.clear();
    }
}

// static funcs
impl Interpolator {
    fn slope_of(first: Event, second: Event) -> f64 {
        (first.value - second.value) / (first.timestamp - second.timestamp) as f64
    }

    fn rounds_to_zero(val: f64) -> bool {
        val.abs() < 0.5
    }
}
