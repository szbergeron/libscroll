use std::collections::VecDeque;
//use std::cell::RwLock;
use std::borrow::Borrow;

type Timestamp = u64;
type Time = f64; // arbitrary units, expressed in fractions of how many TIMESTEP_MILLIS elapse
type TimeDelta = f64;
type TimeDeltaMicros = u64;

type Velocity = f64;
type Position = f64;

use crate::CONFIG as sconfig;

/*const TICKS_PER_TIMUNIT: f64 = 0.5;

const MILLIS_PER_FRAME_DEFAULT: u64 = 16;
//const TIMESTEP_MILLIS: f64 = 0.1;
const TIMESTEP: f64 = 0.1;

const MIN_VELOCITY_TO_IDLE: f64 = 0.002;

const EVENT_EXPIRY_COUNT: usize = 20;
const SAMPLE_EXPIRY_COUNT: usize = 20;

const TICKS_TO_COAST: f64 = 1.6;

const FLIPS_TO_IDLE: u64 = 20;

const POST_ACCEL_SCALE_VELOCITY: f64 = 19.0;
const PRE_ACCEL_SCALE_VELOCITY: f64 = 10.0;

const SHIFT_WINDOW_MS: f64 = 0.0;

const COAST_MIN_VELOCITY: f64 = 0.01;

const BOUNCE_REDUCTION_EXPONENT: f64 = 1.5;
const BOUNCE_REDUCTION_COEFFICIENT: f64 = 1.0;

const OVERSCROLL_ELASTICITY_COEFFICIENT: f64 = 1.0;

const CONTENT_MASS_VALUE: f64 = 6000.0;
const OVERSCROLL_SPRING_CONSTANT: f64 = 0.4;

const BOUNCE_DAMP_FACTOR: f64 = 0.9974;

const MAX_MS_WITHOUT_ZERO_INJECTION: f64 = 150.0;

const MULTIPLY_FIRST_EVENT: f64 = 500.0;

const FIRST_EVENT_SLOPE: f64 = -0.8;*/

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Inactive,
    Interpolating,
    Released(Time), // the velocity and time the release was done at
}

enum TrackPosition {
    Top,
    Bottom,
}

enum BounceState {
    Bouncing(TrackPosition),
    Normal,
}

#[derive(Clone, Copy, Debug, PartialOrd)]
struct Event {
    time: Time, // microseconds since interpolator init
    //delta: f64, // distance represented by this event
    value: f64, // the current absolute "position" of the event
    //bezier_forward: Option<bezier::Curve<geo::Coord2>>,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {})", self.time, self.value)
    }
}

impl std::cmp::Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let cmp_time = self.time.partial_cmp(&other.time).expect("NaN in time field of an event");

        match cmp_time {
            std::cmp::Ordering::Equal => self.value.partial_cmp(&other.value).expect("NaN in value field of an event"),
            other => other
        }
    }
}

impl std::cmp::PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.time == other.time
    }
}

impl std::cmp::Eq for Event {}

/*impl std::cmp::PartialOrd for Event {
    fn partial_cmp(&self*/

#[derive(Clone, Copy)]
struct Sample {
    time: Time,
    velocity: Velocity,
    position: Position,
}

impl std::fmt::Display for Sample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {}, {})", self.time, self.position, self.velocity)
    }
}


pub struct Interpolator {
    redistributable: bool,
    // need record of samplings, matched with timestamps or
    // marked redistributable according to event/frame ratio
    //events: RangedMap<Timestamp, Event>,
    //samples: Vec<Timestamp>,
    
    events: VecDeque<Event>,
    samples: VecDeque<Sample>,
    pan_start_time: Time,
    current_phase: Phase,

    // upper and lower bounds of the track, used for calculating where bouncing happens
    track_bound_upper: f64,
    track_bound_lower: f64,
    track_initial_pos: f64,

    bouncing: BounceState,

    min_tick_period: TimeDelta,

    last_value: f64,
    flips_same_value: u64,
    source: crate::Source,
    //events_y: RangedMap<Timestamp, Event>,

}

impl Interpolator {
    pub fn set_source(&mut self, source: crate::Source) {
        //println!("Sets source to {:?}", source);
        self.source = source;
    }

    pub fn print_events(&self) {
        return;

        println!("Current events are {{{}}}",
                 self.events.iter().fold(
                     String::new(), |s, evt| { s.to_owned() + &evt.to_string()[..] }));
    }

    pub fn new(redistributable: bool, track_bounds: (f64, f64), initial_position: f64) -> Interpolator {
        Interpolator {
            redistributable,
            events: VecDeque::with_capacity(5),
            samples: VecDeque::new(),
            pan_start_time: 0.0,
            min_tick_period: f64::INFINITY,
            current_phase: Phase::Inactive,
            track_bound_lower: track_bounds.0,
            track_bound_upper: track_bounds.1,
            track_initial_pos: initial_position,
            flips_same_value: 0,
            last_value: 0.0,
            bouncing: BounceState::Normal,
            source: crate::Source::Undefined,
        }
    }

    pub fn sample(&mut self, time: Time) -> Position {
        let config = sconfig.read().unwrap();

        self.prevent_coast(time);
        //let time = time + 33.0;

        //let velocity = self.sample_velocity(time);

        let last_sample = self.samples.back().map(|&evt| evt).unwrap_or(Sample { time, velocity: 0.0, position: 0.0 });

        /*for step in last_sample.timestamp..timestamp {
        }*/

        //let time_range = timestamp - last_sample.timestamp;
        let iter = iter_range(last_sample.time + config.SHIFT_WINDOW_MS, time + config.SHIFT_WINDOW_MS, config.TIMESTEP);

        //let mut pos_delta = 0.0;

        let mut cur_position = last_sample.position;
        let mut cur_velocity = last_sample.velocity;
        //println!("Starts {} {}", cur_position_step, cur_velocity_step);
        for (start, end) in iter {
            let stepped_velocity = self.step_velocity(start, end, cur_position, cur_velocity);
            //let approx_velocity = (stepped_velocity + cur_velocity_step) / 2.0;

            let velocity_per_step = (stepped_velocity + cur_velocity) / config.TIMESTEP; // TODO: for now

            //println!("Velocity approx samples to {}", approx_vel);

            let time_delta = end - start;

            let integral = time_delta * velocity_per_step;


            cur_position += integral;
            cur_velocity = stepped_velocity;
            /*println!("Integrates over {}, {} with approx_vel {} to reach {}",
                     start, end, approx_vel, integral);*/
        }
        //println!("Ends {} {}", cur_position_step, cur_velocity_step);
        //println!("Minticks is {}", self.min_tick_period);

        /*if cur_position > self.track_bound_upper && cur_velocity > 0.0 {
            println!("Clamps upper bound, bound is {} and cur_position is {}", self.track_bound_upper, cur_position);
            println!("Upper bound is {}", self.track_bound_upper);
            cur_position = self.track_bound_upper;
            cur_velocity = 0.0;
        } else if cur_position < self.track_bound_lower && cur_velocity < 0.0 {
            println!("Clamps lower bound, bound is {} and cur_position is {}", self.track_bound_lower, cur_position);
            println!("Upper bound is {}", self.track_bound_upper);
            cur_position = self.track_bound_lower;
            cur_velocity = 0.0;
        }*/

        self.samples.push_back(Sample { time, velocity: cur_velocity, position: cur_position });

        //self.check_idle(position);

        self.cull();
        //self.check_idle(cur_position);
        self.check_idle(cur_position, cur_velocity);

        /*if position.is_nan() {
            panic!("Was going to return NaN position");
        }*/
        //println!("Samples at {}, gets ({}, {})", time, cur_position, cur_velocity);

        /*if self.events.len() >= 2 {
            //cur_position
        } else {
            //cur_position + self.short_circuit_single_event()
        }*/

        cur_position
    }

    pub fn cull(&mut self) {
        let config = sconfig.read().unwrap();

        while self.samples.len() > config.SAMPLE_EXPIRY_COUNT {
            self.samples.pop_front();
        }
        while self.events.len() > config.EVENT_EXPIRY_COUNT {
            self.events.pop_front();
        }
    }

    pub fn signal_fling(&mut self, time: Time) {
        //println!("Fling at {}", time);
        self.current_phase = Phase::Released(time);

        //self.flush(time);
    }

    pub fn signal_interrupt(&mut self, time: Time) {
        println!("Interrupt at {}", time);
        //panic!("Interrupt not impl");
        self.current_phase = Phase::Inactive;
        self.flush(time);
        self.min_tick_period = f64::INFINITY;
    }

    pub fn signal_pan(&mut self, time: Time, delta: f64) {
        //println!("Signal pan at {} for {}", time, delta);
        if time == 0.0 {
            panic!("can't pass zero timestamps into signal_pan");
        }
        //println!("Push pan");
        self.current_phase = Phase::Interpolating;

        let (prev_val, prev_time) = self.events.back().map(|evt| (evt.value, evt.time)).unwrap_or((self.track_initial_pos, f64::NEG_INFINITY));

        let current_val = prev_val + delta;

        if time - prev_time <= 0.0 {
            // some events got bunched up, redistribute prior event halfway between current and
            // prev-prev event
            /*println!("Unbunching data");

            let redistributed = self.events.pop_back().unwrap();

            let prev_prev = self.events.back().map(|&evt| evt.clone()).unwrap_or(Event { value: f64::NAN, time: time - TIMESTEP * 2.0 });

            let redis_time = prev_prev.time + (time - prev_prev.time) / 2.0;

            self.events.push_back(Event { value: redistributed.value, time: redis_time });

            self.samples.retain(|s| s.time < prev_prev.time); // invalidate samples that relied on old data*/

            // don't redistribute, just set new.
            // least disruptive behavior here is to drop the old event,
            // and enqueue the new one
            //
            // also remove any samples that rely on the bad data
            self.samples.retain(|s| s.time < prev_time);
            self.events.pop_back();
        } else {
            self.min_tick_period = time - prev_time;
        }

        self.events.push_back(Event { value: current_val, time });
    }

    pub fn animating(&self) -> bool {
        let r = match self.current_phase {
            Phase::Inactive => false,
            _ => true,
        };

        //println!("Says animating is {}", r);

        r
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
    fn flush(&mut self, time: Time) {
        //self.last_interpolated_velocity = self.interpolate(time);
        //self.events.clear();
        //self.samples.clear(); need samples to continue animating
    }

    fn check_idle(&mut self, position: Position, velocity: Velocity) {
        let config = sconfig.read().unwrap();

        match self.current_phase {
            Phase::Released(_) => {
                if position == self.last_value || velocity.abs() < config.MIN_VELOCITY_TO_IDLE {
                    self.flips_same_value += 1;
                } else {
                    self.flips_same_value = 0;
                }
                self.last_value = position;

                if self.flips_same_value > config.FLIPS_TO_IDLE {
                    eprintln!("Goes to idle");
                    println!("check_idle goes to Inactive");
                    self.current_phase = Phase::Inactive;
                }
            },
            Phase::Interpolating => {
                self.flips_same_value = 0;
            },
            Phase::Inactive => {}
        }
    }
    /*fn check_idle(&mut self, velocity: Velocity) {
        if velocity < MIN_VELOCITY_TO_IDLE {
            println!("Libscroll idles with velocity {}", velocity);
            self.current_phase = Phase::Inactive;
        }
    }*/

    fn prevent_coast(&mut self, time: Time) {
        let config = sconfig.read().unwrap();

        match self.current_phase {
            Phase::Interpolating => match self.events.len() {
                0 => {}
                _ => {
                    let evt = self.events.back().expect("Events was empty despite len > 0");
                    let delta = (time - evt.time).abs();
                    if delta > self.min_tick_period * config.TICKS_TO_COAST || delta > config.MAX_MS_WITHOUT_ZERO_INJECTION {
                        // inject event
                        //println!("\n\n\n\n\nCLAMPs velocity to prevent coast");
                        println!("Clamps to prevent coast. Delta {} evt {} min_tick_period {}", delta, evt, self.min_tick_period);
                        //self.current_phase = Phase::Inactive;
                        self.signal_interrupt(time);
                    }
                }
            },
            _ => {},
        }
    }

    fn interpolate(&self, time: Time) -> Velocity {
        let config = sconfig.read().unwrap();

        let first_before = self
            .events
            .iter()
            .filter(|evt| evt.time < time)
            // round is ok since timestamps have no greater enqueing precision than integrals
            .max_by(|evt_a, evt_b| (evt_a.time as u64).cmp(&(evt_b.time as u64)));

        let first_after = self
            .events
            .iter()
            .filter(|evt| evt.time >= time)
            .min_by(|evt_a, evt_b| (evt_a.time as u64).cmp(&(evt_b.time as u64)));

        let second_before = match first_before {
            None => None,
            Some(first) => {
                self
                    .events
                    .iter()
                    .filter(|evt| evt.time < first.time)
                    .max_by(|evt_a, evt_b| (evt_a.time as u64).cmp(&(evt_b.time as u64)))
            }
        };

        let second_after = match first_after {
            None => None,
            Some(first) => {
                self
                    .events
                    .iter()
                    .filter(|evt| evt.time < first.time)
                    .min_by(|evt_a, evt_b| (evt_a.time as u64).cmp(&(evt_b.time as u64)))
            }
        };

        //let events: Vec<&Event> = vec![&first_before, &first_after, &second_before, &second_after]
        let events: Vec<&Event> = vec![&second_before, &first_before, &first_after]
            .into_iter()
            .filter_map(|&evt| evt) // trim Nones
            .collect();

        let result = match events.len() {
            0 => {
                //println!("Interpolate returns 0 as no events exist");
                0.0
            },
            1 => {
                //println!("Interpolate returns 0, as can't get slope of single event");
                let evt = events.first().expect("no elements in size 1 vec");
                //evt.value * MULTIPLY_FIRST_EVENT / (time - evt.time + 1.0)
                //FIRST_EVENT_SLOPE * (time - evt.time) + evt.value
                evt.value / config.MAX_MS_WITHOUT_ZERO_INJECTION
            }
            //1 => Self::interpolate_constant(&events, time),
            2 => Self::interpolate_linear(&events, time),
            3 => Self::interpolate_linear_averaging(&events, time),
            4 => Self::interpolate_hermite(&events, time),
            _ => panic!("Programming error: events len greater than 4"),
        };

        if events.len() == 1 {
            //println!("Sampling imprecisely, only one event available");
        }

        if result == 0.0 {
            //println!("interpolate returned zero. Events vec is {:?} and time is {}. All events is {:?}", events, time, self.events);
        }

        //println!("Interpolates result {} with evt count {}", result, events.len());

        result
    }

    fn outside_bounds(&self, position: Position) -> bool {
        let config = sconfig.read().unwrap();

        position > self.track_bound_upper || position < self.track_bound_lower
    }

    fn short_circuit_single_event(&self) -> Position /* delta */ {
        let config = sconfig.read().unwrap();

        self.events.back().map(|evt| evt.value).unwrap_or(0.0)
    }

    fn fling_boost(&self, velocity: Velocity) -> Velocity {
        let config = sconfig.read().unwrap();

        velocity * config.FLING_BOOST_CONSTANT_FACTOR
    }

    fn handle_overscroll(&self, start: Time, end: Time, position: Position, velocity: Velocity) -> Velocity {
        let config = sconfig.read().unwrap();

        if self.outside_bounds(position) {
            //velocity.abs().powf(0.6).copysign(velocity)
            if self.source.overscrolls() {
                let outside_by = if position > self.track_bound_upper {
                    if velocity < 0.0 {
                        return velocity;
                    }

                    position - self.track_bound_upper
                } else {
                    if velocity > 0.0 {
                        return velocity;
                    }

                    self.track_bound_lower - position
                };

                let abs_vel = velocity.abs();
                let timedelta = end - start;
                let r_velocity = velocity * (1.0 / (outside_by * config.OVERSCROLL_ELASTICITY_COEFFICIENT));

                /*if reduction_amount < 0.0 {
                    panic!("Reduction amount negative");
                } else if reduction_amount > abs_vel {
                    panic!("Overscroll accelerated velocity");
                }*/

                //(abs_vel - reduction_amount).copysign(velocity)
                if r_velocity.is_nan() {
                    panic!("handle_overscroll tried to return NaN");
                }
                r_velocity
            } else {
                if velocity < 0.0 && position < self.track_bound_upper {
                    0.0
                } else if velocity > 0.0 && position > self.track_bound_lower {
                    0.0
                } else {
                    velocity
                }
            }
        } else {
            velocity
        }
    }

    fn accelerate(&self, velocity: Velocity) -> Velocity {
        let config = sconfig.read().unwrap();
        //velocity
        if self.source.accelerates() {
            (velocity / config.ACCEL_DECEL_DESCRIMINANT).abs().powf(config.ACCELERATION_EXPONENT).copysign(velocity) * config.ACCEL_DECEL_DESCRIMINANT
        } else {
            velocity
        }
    }

    fn pre_scale(&self, velocity: Velocity) -> Velocity {
        let config = sconfig.read().unwrap();

        velocity * config.PRE_ACCEL_SCALE_VELOCITY
    }

    fn post_scale(&self, velocity: Velocity) -> Velocity {
        let config = sconfig.read().unwrap();

        velocity * config.POST_ACCEL_SCALE_VELOCITY
    }

    fn decay(&self, start: Time, end: Time, _position: Position, old_velocity: Velocity) -> Velocity {
        if old_velocity.is_nan() {
            panic!("given NaN velocity");
        }

        if old_velocity == 0.0 {
            return 0.0;
        }

        let timedelta = end - start;
        //println!("DECAY: {}, {}", timedelta, old_velocity);
        let abs_vel = old_velocity.abs();

        if timedelta < 0.0 {
            panic!("Negative timedelta passed to decay");
        }

        //let slope = -0.00003 / (old_velocity.log2() + 1.0);
        let friction_factor = if old_velocity != 0.0 {
            old_velocity.abs().powf(1.3) / old_velocity.abs()
        } else {
            0.0
        };
                
        let slope = -0.00009 * friction_factor;
        let new_vel = abs_vel + slope * timedelta;

        let floored = if new_vel < 0.0 {
            0.0
        } else {
            new_vel.copysign(old_velocity)
        };

        //println!("PRODUCES {}", r);

        if floored.abs() > old_velocity.abs() {
            panic!("Somehow accelerated");
        }

        if floored.is_nan() {
            panic!("tried to return NaN velocity. Given {}", old_velocity);
        }

        floored
        //0.0


        /*if velocity == 0.0 {
            panic!();
        }*/
        //velocity * 0.9999
        /*let slope = 0.99;
        let slope = match velocity < 0.0 {
            true => -slope,
            false => slope,
        };

        let velocity = velocity + slope * timedelta;

        if velocity < 0.0 {
            0.0
        } else {
            velocity
        }*/
    }

    fn bounce(&mut self, start: Time, end: Time, position: Position, old_velocity: Velocity) -> Velocity {
        let config = sconfig.read().unwrap();

        if self.outside_bounds(position) {
            let trackposition = if position > self.track_bound_upper {
                TrackPosition::Bottom
            } else {
                TrackPosition::Top
            };

            self.bouncing = BounceState::Bouncing(trackposition);
        }

        match &self.bouncing {
            BounceState::Normal => old_velocity,
            BounceState::Bouncing(trackposition) => {
                if old_velocity.is_nan() {
                    panic!("Given NaN velocity");
                }

                let displacement = match trackposition {
                    TrackPosition::Top => position - self.track_bound_lower,
                    TrackPosition::Bottom => position - self.track_bound_upper,
                };

                let force = -displacement * config.OVERSCROLL_SPRING_CONSTANT;
                let timedelta = end - start;
                let acceleration = force / config.CONTENT_MASS_VALUE;
                let velocity = old_velocity + acceleration * timedelta;

                let velocity = velocity * config.BOUNCE_DAMP_FACTOR;

                if velocity.is_nan() {
                    panic!("Velocity was NaN");
                } else if velocity.is_infinite() {
                    panic!();
                }

                velocity
            }
        }
    }

    fn sample_velocity(&self, start: Time, end: Time) -> Velocity {
        let p1 = self.interpolate(start);
        let p2 = self.interpolate(end);
        let timedelta = end - start;
        let vel = (p2 - p1) * timedelta; // units of TIMESTEP
        //println!("Velocity avg ({}, {}) to from ({}, {}) becomes {}", start, end, p1, p2, vel);

        //println!("\n\n\n\nGives velocity {}", vel);
        vel
    }

    /// provide an approximation of the average velocity after the given time period
    fn step_velocity(&mut self, start: Time, end: Time, position: Position, old_velocity: Velocity) -> Velocity {
        match self.current_phase {
            Phase::Released(release_time) if release_time < start && release_time >= end => {
                let r = self.fling_boost(old_velocity);

                r
            },
            Phase::Released(release_time) if release_time < start => {
                let b = self.bounce(
                    start,
                    end,
                    position,
                    old_velocity);
                let r = self.decay(
                    start,
                    end,
                    position,
                    b);

                /*if r.abs() > old_velocity.abs() {
                    panic!("Velocity increased during release");
                }*/

                r
            },
            Phase::Interpolating | Phase::Released(_) => {
                // short circuit velocity measurement, velocity is just the accelerated
                // interpolation velocity
                self.bouncing = BounceState::Normal;
                let r = self.post_scale(
                            self.handle_overscroll(
                                start,
                                end,
                                position,
                                self.accelerate(
                                    self.pre_scale(
                                        self.sample_velocity(start, end)))));
                r
            },
            Phase::Inactive => 0.0
        }
    }

    fn set_inactive(&mut self) {
        println!("set_inactive sets Inactive");
        self.current_phase = Phase::Inactive;
        self.bouncing = BounceState::Normal;
        self.samples.clear();
    }
}

// static funcs
impl Interpolator {
    fn sample_linear(first: &Event, second: &Event, sample: Time) -> f64 {
        let slope = Self::slope_of(first, second);

        slope * (sample - first.time) + first.value
    }

    fn slope_of(first: &Event, second: &Event) -> f64 {
        //println!("Slope_of asked to compute {}, {}", first, second);
        if first.time == second.time {
            0.0
        } else {
            (first.value - second.value) / ((first.time as i64) - (second.time as i64)) as f64
        }
    }

    fn rounds_to_zero(val: f64) -> bool {
        val.abs() < 0.5
    }

    fn interpolate_constant(events: &Vec<&Event>, at: Time) -> f64 {
        let vel = events.first().expect("interpolate_constant given empty events vec").value;

        //println!("Interpolating constant, gives {}", vel);

        vel
    }

    fn interpolate_linear(events: &Vec<&Event>, at: Time) -> f64 {
        //println!("Interpolating linear");
        let first = events[0];
        let second = events[1];

        if first == second {
            panic!("interpolate_linear given single event");
        }

        /*if first.value == second.value && first.value != 0.0 {
            println!("Equal values in events");
        }*/

        Self::sample_linear(first, second, at)
    }

    fn interpolate_linear_averaging(events: &Vec<&Event>, at: Time) -> f64 {
        //println!("Interpolating 3avg");
        let first = events[1];
        let second = events[2];
        Self::interpolate_linear(&vec![first, second], at)
        /*let mut events = events.clone();
        events.sort();
        let e1 = events[0];
        let e2 = events[1];
        let e3 = events[2];

        let s1 = Self::interpolate_linear(&vec![e1, e2], at);
        let s2 = Self::interpolate_linear(&vec![e2, e3], at);

        let avg = (s1 + s2) / 2.0;

        avg*/
    }

    fn interpolate_hermite(events: &Vec<&Event>, at: Time) -> f64 {
        println!("Interpolating hermite");
        panic!("not implemented");
        0.0
    }
}

struct TimestampIterator {
    end: f64,
    cur: f64,
    step: f64,
}

//impl Iterator<(u64, u64)> for RangeIter 
impl Iterator for TimestampIterator {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            None
        } else if (self.end - self.cur) < self.step {
            let r = (self.cur, self.end);
            self.cur = self.end;
            Some(r)
        } else {
            let r = (self.cur, self.cur + self.step);
            self.cur += self.step;
            Some(r)
        }
    }
}

fn iter_range(start: f64, end: f64, by: f64) -> TimestampIterator {
    TimestampIterator {
        end,
        cur: start,
        step: by,
    }
}
