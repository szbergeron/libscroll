use crate::AxisVector;
use crate::Axis;
use flo_curves::*;
use crate::ranged_map::*;

type Timestamp = u64;
type TimeDeltaMicros = u64;

/// This crate only properly handles events that are uniformly redistributable.
/// Redistributability should only be set during init

#[derive(Clone, Copy)]
struct Event {
    timestamp: Timestamp, // microseconds since interpolator init
    delta: f64, // distance represented by this event
    bezier_forward: Option<bezier::Curve<geo::Coord2>>,
}

struct Interpolator {
    redistributable: bool,
    // need record of samplings, matched with timestamps or
    // marked redistributable according to event/frame ratio
    events: RangedMap<Timestamp, Event>,
    //events_y: RangedMap<Timestamp, Event>,

}

impl Interpolator {
    pub fn new(redistributable: bool) -> Interpolator {
        Interpolator {
            redistributable,
            events: RangedMap::new(),
            //events_y: RangedMap::new(),
        }
    }

    pub fn add_event(&mut self, /*axis: Axis,*/ timestamp: Timestamp, delta: f64) {
        /*match axis {
            Axis::Vertical => self.events_y.insert(Event { timestamp, delta, bezier_forward: None }),
            Axis::Horizontal => self.events_x.insert(Event { timestamp, delta, bezier_forward: None }),
        }*/
        self.events.insert(Event { timestamp, delta, bezier_forward: None });
    }

    /*pub fn sample(timestamp: u64, delta_since: u64) -> AxisVector<f64> {
        //
    }*/

    /*pub fn prepare(start_bound: Timestamp, end_bound: Timestamp) {
        // find events within and directly surrounding bound
    }*/

    pub fn integrate_over(&self, start_bound: Timestamp, end_bound: Timestamp) {
        // Start at end bound, see if neighbors go far enough. If not, find neighbors back
        let (cand_start, end) = self.events.get_neighbors_to(end_bound);
    }

    pub fn redistribute(&mut self, most_recent: Timestamp, ratio: f64, frametime: TimeDeltaMicros) {
        if self.redistributable {
            Self::redistribute_from(&mut self.events, most_recent, ratio, frametime);
            //Self::redistribute_from(&mut self.events_y, most_recent, ratio, frametime);
        }
    }

    fn redistribute_from(map: &mut RangedMap<Timestamp, Event>, most_recent: u64, ratio_e_f: f64, frametime: TimeDeltaMicros) {
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
    }
}

impl Event {
    pub fn project_bezier(&self, map: &RangedMap<Timestamp, Event>) -> bezier::Curve<geo::Coord2> {
        let (prev, this, next, next_next) = map.get_2nd_neighbors_to(self.timestamp);

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
}

impl ToKey<Timestamp> for Event {
    fn to_key(&self) -> Timestamp {
        self.timestamp
    }
}
