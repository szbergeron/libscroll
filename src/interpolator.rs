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
    delta: f64, // distance represented by this event
    //bezier_forward: Option<bezier::Curve<geo::Coord2>>,
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
    //events_y: RangedMap<Timestamp, Event>,

}

impl Interpolator {
    pub fn new(redistributable: bool) -> Interpolator {
        let r = Interpolator {
            redistributable,
            delta: 0,
            events: VecDeque::with_capacity(4),
            //events: RangedMap::new(),
            //samples: vec![],
            //events_y: RangedMap::new(),
        };

        for i in 0..3 {
            r.push_back(Event { timestamp: i, delta: 0 });
        }

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
            1 => self.events.front().expect("Event queue was somehow emptied right underneath us").delta,

            // TODO: change to open ended range syntax once it stabilizes
            //have enough info to do a linear projection, maybe more for the 3+ case later
            x if x >= 2 => {
                let latest = self.events.get(0).expect("Unsafe queue modification occurred");
                let prev = self.events.get(1).expect("Unsafe queue modification occurred");
                let slope: f64 = (latest.delta - prev.delta) / (latest.timestamp - prev.timestamp) as f64;
                let timedelta = sample - latest.timestamp;
                
                let delta = slope * timedelta as f64;

                delta + latest.delta
            }
        }
    }

    fn interpolate(&self, sample: Timestamp) -> f64 {
        match self.events.len() {
            0 => panic!("Asked to interpolate on a zero event queue. Is unsafe concurrent modification happening? If not, this function is being misused."),
            1 => self.events.front().expect("Event queue was somehow emptied right underneath us").delta,

            //only have enough info to do linear interpolation, can avoid doing a bunch of work
            //sampling bezier
            //TODO: consider removing this case for improved maintainability if perf is fine
            2 => {
                let latest = self.events.get(0).expect("Unsafe queue modification occurred");
                let prev = self.events.get(1).expect("Unsafe queue modification occurred");
                let slope: f64 = (latest.delta - prev.delta) / (latest.timestamp - prev.timestamp) as f64;
                let timedelta = sample - prev.timestamp;

                let delta = slope * timedelta as f64;

                delta + prev.delta
            },
            _ => {
                let forward = self.events.iter().filter(|event| { event.timestamp >= sample });
                let backward = self.events.iter().rev().filter(|event| { event.timestamp < sample });

                if forward.count() < 1 { panic!("NI: can't interpolate if not between points") }
                if backward.count() < 1 { panic!("NI: can't interpolate if not between points") }

                let forward = self.events.iter().filter(|event| { event.timestamp >= sample });
                let backward = self.events.iter().rev().filter(|event| { event.timestamp < sample });

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

    pub fn add_event(&mut self, /*axis: Axis,*/ timestamp: Timestamp, delta: f64) {
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

        let iter = self.events.iter().rev();
        let second_last_event_timestamp = iter.nth(1).unwrap().timestamp;

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

    pub fn accurate_upper_bound(&self) -> Timestamp {
        self.bezier_forward.unwrap().end_point.x() as u64
    }
}

impl ToKey<Timestamp> for Event {
    fn to_key(&self) -> Timestamp {
        self.timestamp
    }
}
