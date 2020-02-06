use crate::AxisVector;
use crate::Axis;
use flo_curves::*;
use crate::ranged_map::*;
use std::collections::VecDeque;

type Timestamp = u64;
type TimeDeltaMicros = u64;

/// This crate only properly handles events that are uniformly redistributable.
/// Redistributability should only be set during init

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
}

struct Interpolator {
    redistributable: bool,
    // need record of samplings, matched with timestamps or
    // marked redistributable according to event/frame ratio
    //events: RangedMap<Timestamp, Event>,
    //samples: Vec<Timestamp>,
    
    /// Predicted current position relative to the last position when this variable was last
    /// reset
    delta: f64,
    events: VecDeque<Event>,
    samples: VecDeque<Sample>,
    pan_start_time: Timestamp,
    //events_y: RangedMap<Timestamp, Event>,

}

/*struct EventQueue {
    events: VecDeque<Event>,
}*/

/*impl EventQueue {
    get_before(&self, 
}*/

/*impl VecDeque<Event> {
    //
}*/

impl Interpolator {
    pub fn new(redistributable: bool) -> Interpolator {
        let r = Interpolator {
            redistributable,
            delta: 0.0,
            events: VecDeque::with_capacity(5),
            samples: VecDeque::new(),
            pan_start_time: 0,
            //events: RangedMap::new(),
            //samples: vec![],
            //events_y: RangedMap::new(),
        };

        /*for i in 0..3 {
            r.push_back(Event { timestamp: i, delta: 0 });
        }*/

        return r;
    }

    fn delta_of(&self, sample: Timestamp) -> f64 {
        let latest_sample = self.events.front();

        if latest_sample.timestamp < sample {
            // need to do linear projection, since
            // the most recent data we have is in the past.
            //
            // TODO: see if feasible to add curve here based on 3 prior points
            // + concavity comp
            self.project(sample)
        } else {
            // need to do a 3 point biased bezier, try to interpolate value
            self.interpolate(sample)
        }
    }

    fn project(&self, sample: Timestamp) -> f64 {
        match self.events.len() {
            //no data present, best we can do is assume that's how things are for this frame
            0 => 0.0, 

            //only have one event, have to assume delta will stay the same 
            1 => self.events.front().expect("Event queue was somehow emptied right underneath us").value,

            // TODO: change to open ended range syntax once it stabilizes
            //have enough info to do a linear projection, maybe more for the 3+ case later
            x if x >= 2 => {
                let latest = self.events.get(0).expect("Unsafe queue modification occurred");
                let prev = self.events.get(1).expect("Unsafe queue modification occurred");
                let slope: f64 = (latest.value - prev.value) / (latest.timestamp - prev.timestamp) as f64;
                let timedelta = sample - latest.timestamp;
                
                let value = slope * timedelta as f64;

                value + latest.value
            },
            _ => panic!("Project got len that didn't match 0, 1, or >2")
        }
    }

    fn interpolate(&self, sample: Timestamp) -> f64 {
        match self.events.len() {
            0 => panic!("Asked to interpolate on a zero event queue. Is unsafe concurrent modification happening? If not, this function is being misused."),
            //1 => self.events.front().expect("Event queue was somehow emptied right underneath us").delta,
            1 => panic!("Interpolating with a single event makes no real sense here"),

            //only have enough info to do linear interpolation, can avoid doing a bunch of work
            //sampling bezier
            //TODO: consider removing this case for improved maintainability if perf is fine
            2 => {
                let latest = self.events.get(0).expect("Unsafe queue modification occurred");
                let prev = self.events.get(1).expect("Unsafe queue modification occurred");
                let slope: f64 = (latest.value - prev.value) / (latest.timestamp - prev.timestamp) as f64;
                let timedelta = sample - prev.timestamp;

                let value = slope * timedelta as f64;

                value + prev.value
            },
            _ => {
                let forward = self.events.iter().filter(|event| { event.timestamp >= sample });
                let backward = self.events.iter().rev().filter(|event| { event.timestamp < sample });

                if forward.count() < 1 { panic!("NI: can't interpolate if not between points") }
                if backward.count() < 1 { panic!("NI: can't interpolate if not between points") }

                let mut forward = self.events.iter().filter(|event| { event.timestamp >= sample });
                let mut backward = self.events.iter().rev().filter(|event| { event.timestamp < sample });

                let forward_point = forward.next().expect("Forward event buffer is empty");
                let forward_control = match forward.next() {
                    Some(event) => event,
                    None => forward_point // assume next forward point would be same as current forward point
                };

                let backward_point = backward.next().expect("Backward event queue is empty");
                let backward_control = match backward.next() {
                    Some(event) => event,
                    None => backward_point // assume previous point to backward_point is closely related to backward_point
                };

                let curve = Event::project_bezier(*backward_control, *backward_point, *forward_point, *forward_control);

                curve.point_at_pos(sample as f64).y()
            }
            // have at least 3 events, at least 2 of which are in front of the current event.
            /*3 if self.events.iter().filter(|event| { event.>= sample => {
                0.0
            },

            // have enough to do bezier interpolation, but sample is not far enough
            // in the past to have 2 events in front of it,
            // so can't do fully "correct" bezier interpolation
            3 => {
                0.0
            }*/
        }
    }

    fn sample_linear(first: Event, second: Event, sample: Timestamp) -> f64 {
        let slope = Self::slope_of(first, second);

        slope * sample as f64 + first.value
    }

    fn get_x_axis_intercept(first: Event, second: Event) -> Timestamp {
        let slope = Self::slope_of(first, second);

        //0 = slope(?) + first.delta, (-first.delta) / slope = ? relative to first.timestamp

        ((-first.value) / slope) as Timestamp + first.timestamp
    }

    fn slope_of(first: Event, second: Event) -> f64 {
        (first.value - second.value) / (first.timestamp - second.timestamp) as f64
    }

    pub fn add_event(&mut self, /*axis: Axis,*/ timestamp: Timestamp, delta: f64) {
        let event = Event {
            timestamp,
            value: self.events
                .back()
                .map_or(0.0, |event| event.value)
        };
        /*match axis {
            Axis::Vertical => self.events_y.insert(Event { timestamp, delta, bezier_forward: None }),
            Axis::Horizontal => self.events_x.insert(Event { timestamp, delta, bezier_forward: None }),
        }*/
        // first need to find what the predicted integral was supposed to bezier
        //let predicted = integrate_over(events.deref().last_key_value().unwrap().1.timestamp, timestamp);
        // also need to find difference between the predicted bezier profile 
        
        // First need to figure out what the predicted value was supposed to be
        // Also need to go one window back and recreate bezier, then resample any frames from
        // during that period

        let mut error_delta = 0.0;

        //let mut iter = self.events.iter().rev();
        //let second_last_event_timestamp = iter.nth(1).unwrap().timestamp;

        match self.events.len() {
            //any prior samples used a constant 0 sample, can simply add event
            0 => {
                self.start_pan();
                //self.events.push_back(Event { timestamp, delta });
                self.events.push_back(event);
            },
            //we have a prior sample, we can do linear correction to figure out when the swipe
            //started, also there are few enough events we can recalculate the delta in full
            1 => {
                //self.events.push_back(Event { timestamp, delta });

                let mut new_total_delta = 0.0;
                /*for sample in self.samples {
                    new_total_delta += self.interpolate(sample.timestamp);
                }*/

                //need to find x-axis intercept to find when any samples would have to be later
                //than to count as part of the scroll
                let first_event = *self.events.get(1).unwrap();
                let second_event = *self.events.get(0).unwrap();

                //let sample_since = Self::get_x_axis_intercept(first_event, second_event);
                let sample_since = self.pan_start_time;

                let samples: Vec<Timestamp> = self.samples
                    .iter()
                    .filter(|sample| sample.timestamp >= sample_since)
                    .map(|sample| sample.timestamp)
                    .collect();

                for sample in samples {
                    new_total_delta += Self::sample_linear(first_event, second_event, sample);
                }

                self.delta = new_total_delta;
            },
            //we have enough to do a very basic bezier predict between the
            //first three by doing linear projection for the fourth point
            //into the future and past and resampling
            //
            //Also still few enough that we can resample all the points and completely refresh the
            //delta
            _ /* >=2 */ => {
                let cur_real_event = *self.events.get(0).unwrap();
                let prev_real_event = *self.events.get(1).unwrap();

                let prev_prev_event = match self.events .get(2) {
                    Some(&evt) => evt,
                    None => {
                        //let timedelta = cur_real_event.timestamp - prev_real_event.timestamp;
                        //let slope = Self::slope_of(prev_real_event, cur_real_event);
                        Event {
                            timestamp: prev_real_event.timestamp - 1,
                            value: Self::sample_linear(prev_real_event, cur_real_event, prev_real_event.timestamp - 1),
                        }
                    }
                };

                /*let projected_next_event = Event {
                    timestamp: cur_real_event.timestamp + 1,
                    value: Self::sample_linear(prev_real_event, cur_real_event, cur_real_event.timestamp + 1),
                };*/

                //let projected_curve = Event::project_bezier(prev_prev_event, prev_real_event, cur_real_event, projected_next_event);

                //let first_real_event = *self.events.get(2).unwrap();
                //let middle_real_event = *self.events.get(1).unwrap();
                //let last_real_event = *self.events.get(0).unwrap();

                //let sample_since = Self::get_x:axis_intercept(

                let first_timedelta = middle_real_event.timestamp - first_real_event.timestamp;
                let last_timedelta = last_real_event.timestamp - middle_real_event.timestamp;

                let first_fake_event = Event {
                    timestamp: first_real_event.timestamp - first_timedelta,
                    delta: Self::sample_linear(first_real_event, middle_real_event, first_real_event.timestamp - first_timedelta),
                };

                let last_fake_event = Event {
                    timestamp: last_real_event.timestamp + last_timedelta,
                    delta: Self::sample_linear(middle_real_event, last_real_event, last_real_event.timestamp + last_timedelta),
                };

                let first_to_middle_bezier = Event::project_bezier(first_fake_event, first_real_event, middle_real_event, last_real_event);
                let middle_to_last_bezier = Event::project_bezier(first_real_event, middle_real_event, last_real_event, last_fake_event);

                let sample_since = self.pan_start_time;
                let samples: Vec<Timestamp> = self.samples
                    .iter()
                    .filter(|sample| sample.timestamp >= sample_since)
                    .map(|sample| sample.timestamp)
                    .collect();

                let mut new_total_delta = 0.0;
                for sample in samples {
                    if( sample < first_real_event.timestamp ) {
                        new_total_delta += Self::sample_linear(first_real_event, middle_real_event, sample);
                    } else if( sample > last_real_event.timestamp ) {
                        new_total_delta += Self::sample_linear(middle_real_event, last_real_event, sample);
                    } else if( 
                }
            }
        }

        //let projected_region = 

        /*for sample_timestamp in self.samples.iter().rev() {
            if *sample_timestamp < second_last_event_timestamp {
                // this is from before the only bezier that changes, so we can ignore it
                break;
            }

            error_delta -= self.events.get_before(*sample_timestamp).unwrap().bezier_forward.unwrap().point_at_pos(*sample_timestamp as f64).y();
        }*/

        // correct linear part of prediction, if any sample during this time occurred
        //

        //self.events.insert(Event { timestamp, delta, bezier_forward: None });
    }

    pub fn end_pan(&mut self) {
        //self.delta = 0.0;
        self.events.clear();
        self.samples.clear();
    }

    pub fn start_pan(&mut self, starts: Timestamp) {
        self.delta = 0.0;
        self.pan_start_time = starts;
    }


    /*pub fn sample(timestamp: u64, delta_since: u64) -> AxisVector<f64> {
        //
    }*/

    /*pub fn prepare(start_bound: Timestamp, end_bound: Timestamp) {
        // find events within and directly surrounding bound
    }*/

    /*pub fn integrate_over(&self, start_bound: Timestamp, end_bound: Timestamp) -> f64 {
        let points = self.events.get_all_after(start_bound);
        //points.sort_by(|&a, &b| a.1.timestamp.cmp(b.1.timestamp));

        if points.get(points.len() - 1).unwrap().0 > end_bound {
            //no need to do prediction/linear projection
        }
    }*/

    /*pub fn integrate_over(&self, start_bound: Timestamp, end_bound: Timestamp) {
        assert!(start_bound < end_bound, "Couldn't integrate over negative interval");
        // Start at end bound, see if neighbors go far enough. If not, find neighbors back
        let stride = 1; // how many microseconds between each sample
        //let (cand_start, end) = self.events.get_neighbors_to(end_bound);
        let mut start = self.events.get_before(start_bound).unwrap();

        let mut sum = 0.0;

        for num in start_bound..end_bound {
            if num > start.accurate_upper_bound() {
                match self.events.get_after(start.timestamp) {
                    Some(event) => {
                        //start = cand_end;
                        //cand_end = event
                        start = event;
                    },
                    None => panic!("Next event simulation not implemented"),
                }
            }

            sum += start.bezier_forward.unwrap().point_at_pos(num as f64).y();
        }
    }*/

    /*pub fn redistribute(&mut self, most_recent: Timestamp, ratio: f64, frametime: TimeDeltaMicros) {
        if self.redistributable {
            Self::redistribute_from(&mut self.events, most_recent, ratio, frametime);
            //Self::redistribute_from(&mut self.events_y, most_recent, ratio, frametime);
        }
    }*/

    /*fn redistribute_from(map: &mut RangedMap<Timestamp, Event>, most_recent: u64, ratio_e_f: f64, frametime: TimeDeltaMicros) {
        let mut new_timestamp = most_recent;
        let delta_mis = (ratio_e_f * frametime as f64) as TimeDeltaMicros;
        for (timestamp, event) in map.iter_mut().rev() {
            event.timestamp = new_timestamp;
            new_timestamp -= delta_mis;
        }

        for (timestamp, event) in map.iter_mut() {
            let curve = event.project_bezier(map);
            event.bezier_forward = Some(curve);
        }
    }*/
}

impl Event {
    pub fn project_bezier(prev: Event, this: Event, next: Event, next_next: Event) -> bezier::Curve<geo::Coord2> {
        //let (prev, this, next, next_next) = map.get_2nd_neighbors_to(self.timestamp);

        let slope_ctl_first = (this.delta - prev.delta) / (this.timestamp - prev.timestamp) as f64;

        let projected_point = this.delta + slope_ctl_first * (next.timestamp - this.timestamp) as f64;

        let ctl_point_1 = Coord2(next.timestamp as f64, (next.delta + projected_point) / 2.0);

        let slope_ctl_second = (next_next.delta - next.delta) / (next_next.timestamp - next.timestamp) as f64;
        // this time, need to project backward
        let projected_point = next.delta - slope_ctl_second * (next.timestamp - this.timestamp) as f64;

        let ctl_point_2 = Coord2(this.timestamp as f64, (this.delta + projected_point) / 2.0);

        let curve = bezier::Curve::from_points(
            Coord2(this.timestamp as f64, this.delta),
            (ctl_point_1, ctl_point_2),
            Coord2(next.timestamp as f64, next.delta)
        );

        curve
    }

    /*pub fn accurate_upper_bound(&self) -> Timestamp {
        self.bezier_forward.unwrap().end_point.x() as u64
    }*/
}

impl ToKey<Timestamp> for Event {
    fn to_key(&self) -> Timestamp {
        self.timestamp
    }
}
