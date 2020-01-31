/*!
 * This library serves as an event interpretation library.
 * To use, you will need to take the raw events you recieve
 * on your platform and adapt it to a compatible input
 * interface. You are expected to call get_pan() once on every frame.
 * It expects an estimation of the next frametime as well as how
 * long until the current frame will be rendered. This allows overshoot
 * calculation to take place.
 */

extern crate num;
use std::f64;

mod circular_backqueue;

mod interpolator;

mod ranged_map;

use std::ops;

//#[macro_use]
//extern crate smart_default;

// configs

/// Determines how much "smoothing" happens, at a direct cost to responsiveness to an action
/// A large number means more past events will be counted in the current velocity,
/// which avoids skipping over or being "hitched" by anomalies, but also means
/// that changes to velocity after the initial touch are responded to slower
const MAX_EVENTS_CONSIDERED: u32 = 5;

/// Determines whether the prior config (MAX_EVENTS_CONSIDERED) is used.
/// If false, no smoothing occurs and the velocity is simply the most recent event
/// Equivalent to setting MAX_EVENTS_CONSIDERED to 1, but allows a performance shortcut
const ENABLE_VELOCITY_SMOOTHING: bool = true;

const FLING_FRICTION_FACTOR: f64 = 0.998;

const PAN_ACCELERATION_FACTOR_TOUCHPAD: f64 = 1.34;

/// Used to specify over what window (in number of frames) the ratio of input events to frames
/// should be derived. The ratio is then used to interpolate/extrapolate input events
const SAMPLE_OVER_X_FRAMES: usize = 10;

/// The granularity through which displacement is integrated from the velocity function (sampled
/// with velocity_at(f64))
const INTEGRATION_DX: f64 = 0.07;

/// Degree of polynomial used to interpolate velocity
const VELOCITY_POLYNOMIAL_DEGREE: usize = 4;

type Millis = f64;

/// Represents a single scrollview and tracks all state related to it.
#[derive(Default)]
pub struct Scrollview {
    content_height: u64,
    content_width: u64,
    viewport_height: u64,
    viewport_width: u64,

    current_velocity: AxisVector<f64>,
    current_position: AxisVector<f64>,

    current_source: Source,

    frametime: Millis,
    time_to_pageflip: Millis,

    current_timestamp: u64,

    interpolation_ratio: f64,

    input_per_frame_log: circular_backqueue::ForgetfulLogQueue<u32>,

    timer: Option<Box<dyn FnMut() -> f64>>,

    
    // pairing of a (timestamp, magnitude) for pan events
    pan_log_x: circular_backqueue::ForgetfulLogQueue<(u64, f64)>,
    pan_log_y: circular_backqueue::ForgetfulLogQueue<(u64, f64)>,
}

/// Describes a vector in terms of its 2 2d axis magnitudes,
/// used often to describe transforms and offsets
#[derive(Copy)]
#[derive(Clone)]
#[derive(Default)]
pub struct AxisVector<T> where T: num::Num, T: PartialOrd, T: Copy {
    pub x: T,
    pub y: T,

    x_threshold: T,
    y_threshold: T,
    
    decaying: bool,
}

impl<T> AxisVector<T> where T: num::Num, T: PartialOrd, T: Copy {
    fn difference(self, other: AxisVector<T>) -> AxisVector<T> {
        AxisVector {
            x: self.x - other.x,
            y: self.y - other.y,
            ..self
        }
    }

    fn replace(&mut self, axis: Axis, magnitude: T) {
        match axis {
            Axis::Horizontal => self.x = magnitude,
            Axis::Vertical => self.y = magnitude,
        }
    }

    fn get_at(&self, axis: Axis) -> T {
        match axis {
            Axis::Horizontal => self.x,
            Axis::Vertical => self.y
        }
    }

    fn update(&mut self, axis: Axis, magnitude: T) {
        match axis {
            Axis::Horizontal => self.x = magnitude + self.x,
            Axis::Vertical => self.y = magnitude + self.y,
        }
    }
}

impl AxisVector<f64> {
    fn decay_active(&self) -> bool {
        self.decaying && self.x > self.x_threshold && self.y > self.y_threshold
    }

    fn decay_start(&mut self) {
        self.decaying = true;
    }

    fn step_frame(&mut self, device: Source) {
        if self.decay_active() {
            self.x = Scrollview::fling_decay(self.x, device);
            self.y = Scrollview::fling_decay(self.y, device);
        }

        if self.x < self.x_threshold && self.y < self.y_threshold {
            self.decaying = false;
        }
    }

    fn scale(&self, scalar: f64) -> AxisVector<f64> {
        AxisVector {
            x: self.x * scalar,
            y: self.y * scalar,
            ..self.clone()
        }
    }
}

// TODO: consider naming, doing pythagorean add on + may make more sense, with alternative op to
// simply add elems
impl<T> ops::Add<AxisVector<T>> for AxisVector<T> where T: num::Num, T: PartialOrd, T: Copy {
    type Output = AxisVector<T>;

    fn add(self, rhs: AxisVector<T>) -> AxisVector<T> {
        AxisVector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            ..self

        }
    }
}

#[derive(Copy)]
#[derive(Clone)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Pass along with any events to indicate what kind of device the event came from
#[derive(Copy)]
#[derive(Clone)]
pub enum Source {
    /// Device type is unknown, assume nothing (very suboptimal to actually use this, should only
    /// be used when device type can not be feasibly deduced, and even then may not be the best
    /// choice)
    Undefined,
    /// Device is a touchscreen, hint to avoid acceleration, but perform tracking prediction
    Touchscreen,
    /// Device is a touchpad, hint to accelerate input, and perform tracking prediction
    Touchpad,
    /// Device is a mousewheel that reports deltas of around 15 degrees (coarse) and requires
    /// smoothing and mid-delta animation
    Mousewheel,
    /// Device is a mousewheel that reports deltas of less than 15 degrees (usually much less,
    /// indicating that very little/no smoothing needs to be applied)
    PreciseMousewheel,
    /// Do no manual smoothing or acceleration, assume driver already does this or input method
    /// would be strange to use with either
    Passthrough,
    /// Same as passthrough, but input fling events should trigger a kinetic fling animation
    KineticPassthrough,
    /// The device type last used
    Previous,
}

impl Default for Source {
    fn default() -> Self { Source::Undefined }
}

/*pub enum Event {
    Pan { timestamp: u64, axis: Axis, amount: i32 }, // doesn't use AxisVector since some platforms only send one pan axis at once // TODO: consider AxisVector[Optional]
    Fling { timestamp: u64 },
    Interrupt { timestamp: u64 },
    //Zoom?
}*/

// pub interface
impl Scrollview {
    /// Create a new scrollview with default settings
    ///
    /// Warning: these settings are unlikely to be
    /// particularly useful, so set_geometry(), set_avg_frametime(), and any
    /// other relevant initialization functions still need to be used
    pub fn new() -> Scrollview {
        Scrollview {
            input_per_frame_log: circular_backqueue::ForgetfulLogQueue::new(SAMPLE_OVER_X_FRAMES),
            ..Default::default()
        }
    }

    /// Deletes/deinitializes the current scrollview
    ///
    /// Primarily intended for ffi use, Scrollview implements Drop
    /// where deinitialization is required, so this is only useful
    /// for ffi use
    ///
    /// NOTE: likely will be removed, not sure why I put this in here to begin with
    pub fn del(_: Scrollview) {}

    pub fn set_clock_callback<F>(&mut self, cb: F) where F: Fn() -> f64, F: Clone {
        self.timer = Some(Box::new(cb.clone()));
    }

    /// Set the geometry for the given scrollview
    ///
    /// Can be used both on scrollview initialization and on scrollview resize
    pub fn set_geometry(
        &mut self,
        content_height: u64,
        content_width: u64,
        viewport_height: u64,
        viewport_width: u64,
    ) {
        self.content_height = content_height;
        self.content_width = content_width;
        self.viewport_height = viewport_height;
        self.viewport_width = viewport_width;
    }

    /// Add an event to the queue to be processed for the next call to
    /// step_frame()
    /// NOTE: doesn't simplify usage much and hurts ffi interop, so currently exposing the
    /// individual push_... functions instead (impl complexity is similar/same between both
    /// methods)
    /*pub fn push_event(
        &mut self,
        event: &Event
    ) {
        match event {
            Event::Pan { timestamp, axis, amount } => self.push_pan(*timestamp, *axis, *amount),
            Event::Fling {..} => self.push_fling(),
            Event::Interrupt {..} => self.push_interrupt(),
        }
    }*/

    /// True if scrollview should continue to be polled
    /// even in absence of events (fling or other 
    /// animation in progress)
    pub fn animating(&self) -> bool {
        self.current_velocity.decay_active()
    }

    /// Advances scrollview state by a frame,
    /// Serves to step through animations one frame at a time
    ///
    /// After any event, continue to call this on every
    /// page-flip (new frame) until animating() returns false
    pub fn step_frame(&mut self, timestamp: Option<u64>) {
        self.interpolation_ratio = self.input_per_frame_log.all().iter().sum::<u32>() as f64 / self.input_per_frame_log.size() as f64;

        self.current_timestamp = timestamp.unwrap_or(1);

        //self.current_velocity = self.current_velocity.clone().step_frame(&self);
        self.current_velocity.step_frame(self.current_source);

        self.update_velocity();

        // update position with interpolated velocity
        self.current_position.x += Self::accelerate(self.current_velocity.x, self.current_source) * self.interpolation_ratio * self.frametime;
        self.current_position.y += Self::accelerate(self.current_velocity.y, self.current_source) * self.interpolation_ratio * self.frametime;

        self.input_per_frame_log.push(0); // add new frame for events to pile into
    }
    
    /// Should be called at scrollview initialization time.
    /// Will internally flush any active events or animations,
    /// so should only be used when scrollview is inactive or
    /// when absolutely necessary (monitor refresh rate changes)
    ///
    /// Used for position prediction (better pan tracking)
    pub fn set_avg_frametime(&mut self, milliseconds: f64) {
        self.frametime = milliseconds;
    }

    /// Indicate how long there is until the next frame will be rendered
    /// to the screen.
    ///
    /// Used for position prediction (better pan tracking)
    pub fn set_next_frame_predict(&mut self, milliseconds: f64) {
        self.time_to_pageflip = milliseconds;
    }

    /// Get the position of the content's top-left corner relative to
    /// the top-left corner of the viewport
    ///
    /// NOTE: either axis may be negative. This indicates an overscroll is occurring.
    /// Recommended way of handling this is to checkerboard that area visually
    /// and draw true to the provided geometry. This matches platform behavior for OSX and Windows,
    /// as well as some Linux programs, and is often called the "rubber band effect"
    pub fn get_position_absolute(&self) -> AxisVector<f64> {
        self.current_position + self.get_overshoot()
    }

    // Get the position of the content's top-left corner relative to
    // its position before the most recent step_frame(), saying how much
    // the content should be moved from its last position
    //
    // Note: may support in future, but unclear if this provides any benefits currently, and is
    // difficult to support with prediction. Currently not provided.
    /*pub fn get_position_relative(&self) -> AxisVector<f64> {
        self.current_position.difference(self.prior_position)
    }*/

    /// Enqueue a pan event for the referenced scrollview
    pub fn push_pan(&mut self, axis: Axis, amount: f64, timestamp: Option<u64>) {
        match axis {
            Axis::Horizontal => self.pan_log_x.push((timestamp.unwrap_or(self.current_timestamp), amount)),
            Axis::Vertical => self.pan_log_y.push((timestamp.unwrap_or(self.current_timestamp), amount)),
        }
    }

    /// Enqueue a fling (finger lift at any velocity) for the referenced scrollview
    pub fn push_fling(&mut self, _timestamp: Option<u64>) {
        self.current_velocity.decay_start();
    }

    /// Enqueue a scroll interrupt (finger down at any time, gesture start) for the referenced
    /// scrollview
    pub fn push_interrupt(&mut self, _timestamp: Option<u64>) {
        self.pan_log_x.clear();
        self.pan_log_y.clear();
        self.current_velocity = AxisVector { x: 0.0, y: 0.0, ..self.current_velocity };
    }

    /// Set what device type is going to be providing any events that follow until the next source
    /// is declared
    pub fn set_source(&mut self, source: Source) {
        self.current_source = source;
    }
}

// private impl
impl Scrollview {
    fn get_overshoot(&self) -> AxisVector<f64> {
        let time_to_target = (self.frametime / 2.0) + self.time_to_pageflip;

        AxisVector {
            x: self.current_velocity.x * time_to_target,
            y: self.current_velocity.y * time_to_target,
            decaying: false,
            ..Default::default()
        }
    }

    // Uses backlog and input acceleration curve to smooth recent pan deltas
    fn update_velocity(&mut self) {
        if ENABLE_VELOCITY_SMOOTHING == false {
            // last input vector on each axis is unsmoothed velocity
            self.current_velocity = AxisVector {
                x: self.pan_log_x.get_or_avg(0).1,
                y: self.pan_log_y.get_or_avg(0).1,
                ..self.current_velocity
            }
        } else {
            // sum total weights
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;

            // end divisor for calculating weighting
            let mut weight_x = 0.0;
            let mut weight_y = 0.0;
            
            let axes = vec![(&self.pan_log_x, &mut sum_x, &mut weight_x), (&self.pan_log_y, &mut sum_y, &mut weight_y)];

            // need to do weighted averages
            for (log, sum, weight) in axes {
                for i in 0..(MAX_EVENTS_CONSIDERED - 1) {
                    match log.get(i as usize) {
                        None => (),
                        Some((timestamp, magnitude)) => {
                            let staleness = self.current_timestamp - timestamp;
                            let staleness_mult_factor = 1.0 / (staleness as f64);

                            *weight += staleness_mult_factor;

                            *sum += magnitude * staleness_mult_factor;
                        }
                    }
                }
            }

            let avg_x = sum_x / weight_x;
            let avg_y = sum_y / weight_y;

            self.current_velocity = AxisVector {
                x: avg_x,
                y: avg_y,
                ..self.current_velocity
            }
        }
    }

    /// Returns the displacement over a range on (x, y) as AxisVector
    ///
    /// NOTE: currently a placeholder while polynomial interpolation is added
    fn integrate(&self, start: f64, end: f64) -> AxisVector<f64> {
        self.current_velocity.scale(end - start)
    }

    /// Returns the velocity at a given time as (x, y) as AxisVector
    fn velocity_at(&self, time: f64) -> AxisVector<f64> {
        self.current_velocity
    }

    // TODO: move to pref
    fn accelerate(from: f64, device: Source) -> f64 {
        match device {
            Source::Touchpad => from.powf(PAN_ACCELERATION_FACTOR_TOUCHPAD),
            _ => from,
        }
    }

    // should be changed later to allow different curves, 
    fn fling_decay(from: f64, device: Source) -> f64 {
        match device {
            Source::Touchpad | Source::KineticPassthrough => from.powf(FLING_FRICTION_FACTOR),
            _ => 0.0,
        }
    }
}
