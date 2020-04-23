#![feature(map_first_last)]
#![feature(half_open_range_patterns)]

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

mod interpolate;

mod ranged_map;

use std::ops;
use interpolate::Interpolator;

type Timestamp = u64;

use std::fs::File;
use std::io::prelude::*;

//#[macro_use]
//extern crate smart_default;

// configs

//// Determines how much "smoothing" happens, at a direct cost to responsiveness to an action
//// A large number means more past events will be counted in the current velocity,
//// which avoids skipping over or being "hitched" by anomalies, but also means
//// that changes to velocity after the initial touch are responded to slower
//const MAX_EVENTS_CONSIDERED: u32 = 5;

//// Determines whether the prior config (MAX_EVENTS_CONSIDERED) is used.
//// If false, no smoothing occurs and the velocity is simply the most recent event
//// Equivalent to setting MAX_EVENTS_CONSIDERED to 1, but allows a performance shortcut
//const ENABLE_VELOCITY_SMOOTHING: bool = true;

//const FLING_FRICTION_FACTOR: f64 = 0.998;

//const PAN_ACCELERATION_FACTOR_TOUCHPAD: f64 = 1.34;

//// Used to specify over what window (in number of frames) the ratio of input events to frames
//// should be derived. The ratio is then used to interpolate/extrapolate input events
const SAMPLE_OVER_X_FRAMES: usize = 10;

const DEBUG: bool = false;

const VALUE_MULTIPLIER: f64 = 6.0;

//// The granularity through which displacement is integrated from the velocity function (sampled
//// with velocity_at(f64))
//const INTEGRATION_DX: f64 = 0.07;

//// Degree of polynomial used to interpolate velocity
//const VELOCITY_POLYNOMIAL_DEGREE: usize = 4;

//type Millis = f64;

/// Represents a single scrollview and tracks all state related to it.
//#[derive(Default)]
pub struct Scrollview {
    content_height: f64,
    content_width: f64,
    viewport_height: f64,
    viewport_width: f64,

    current_source: Source,

    dbg_amt_x: f64,
    dbg_amt_y: f64,

    //frametime: Millis,
    //time_to_pageflip: Millis,

    //current_timestamp: u64,

    //interpolation_ratio: f64,

    input_per_frame_log: circular_backqueue::ForgetfulLogQueue<u32>,

    x: Interpolator,
    y: Interpolator,
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
    /*fn difference(self, other: AxisVector<T>) -> AxisVector<T> {
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
    }*/
}

impl AxisVector<f64> {
    pub fn scale(&self, scalar: f64) -> AxisVector<f64> {
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

impl<T> std::fmt::Display for AxisVector<T>
    where T: std::fmt::Display, T: num::Num, T: PartialOrd, T: Copy
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Pass along with any events to indicate what kind of device the event came from
#[derive(Copy, Clone, Debug)]
//#[derive(Clone)]
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

// pub interface
impl Scrollview {
    /// Gives the current best estimate for the position of the content relative to
    /// the viewport in device pixels
    pub fn sample(&mut self, timestamp: Timestamp) -> AxisVector<f64> {
        if !DEBUG {
            AxisVector {
                //x: self.x.sample(timestamp as f64) * VALUE_MULTIPLIER,
                x: 0.0,
                y: self.y.sample(timestamp as f64),
                ..Default::default()
            }
        } else {
            AxisVector {
                x: self.dbg_amt_x,
                y: self.dbg_amt_y,
                ..Default::default()
            }
        }
    }
    /// Create a new scrollview with default settings
    ///
    /// Warning: these settings are unlikely to be
    /// particularly useful, so set_geometry(), set_avg_frametime(), and any
    /// other relevant initialization functions still need to be used
    pub fn new() -> Scrollview {
        Scrollview {
            input_per_frame_log: circular_backqueue::ForgetfulLogQueue::new(SAMPLE_OVER_X_FRAMES),
            content_height: 0.0,
            content_width: 0.0,
            viewport_height: 0.0,
            viewport_width: 0.0,
            current_source: Source::Undefined,
            dbg_amt_y: 0.0,
            dbg_amt_x: 0.0,
            //frametime: 0.0,
            //time_to_pageflip: 0.0,
            //current_timestamp: 0,
            //interpolation_ratio: 0.0,
            x: Interpolator::new(false, (0.0, 0.0), 0.0),
            y: Interpolator::new(false, (0.0, 0.0), 0.0),
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

    /// Set the geometry for the given scrollview
    ///
    /// Can be used both on scrollview initialization and on scrollview resize
    pub fn set_geometry(
        &mut self,
        content_height: f64,
        content_width: f64,
        viewport_height: f64,
        viewport_width: f64,
    ) {
        self.content_height = content_height;
        self.content_width = content_width;
        self.viewport_height = viewport_height;
        self.viewport_width = viewport_width;

        self.x.set_geometry(0.0, (content_width - viewport_width) as f64);
        self.y.set_geometry(0.0, (content_height - viewport_height) as f64);
    }

    /// True if scrollview should continue to be polled
    /// even in absence of events (fling or other 
    /// animation in progress)
    pub fn animating(&self) -> bool {
        println!("Animating called to check in");

        /*let mut file = File::open("/home/sawyer/ctl1.txt").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();*/
        //println!("contents are {}", contents);
        //let b: f64 = contents.parse().unwrap();
        //println!("interprets {}", b);

        //return b != 0.0;
        //return contents.len() > 0;
        //
        //self.current_velocity.decay_active()
        self.x.animating() || self.y.animating()

        //true
    }

    /// Enqueue a pan event for the referenced scrollview
    pub fn push_pan(&mut self, axis: Axis, amount: f64, timestamp: Option<u64>) {
        println!("push_pan with {:?}, {}, {}", axis, amount, timestamp.unwrap());
        if !DEBUG {
            match axis {
                Axis::Horizontal => {},
                //Axis::Horizontal => self.x.signal_pan(timestamp.unwrap() as f64, amount),
                Axis::Vertical => self.y.signal_pan(timestamp.unwrap() as f64, amount),
            }
        } else {
            match axis {
                Axis::Horizontal => self.dbg_amt_x += amount,
                Axis::Vertical => self.dbg_amt_y += amount,
            }
        }
    }

    /// Enqueue a fling (finger lift at any velocity) for the referenced scrollview
    pub fn push_fling(&mut self, timestamp: Option<u64>) {
        println!("push_fling with {}", timestamp.unwrap());
        //self.current_velocity.decay_start();
        //self.x.signal_fling(timestamp.unwrap() as f64);
        self.y.signal_fling(timestamp.unwrap() as f64);
    }

    /// Enqueue a scroll interrupt (finger down at any time, gesture start) for the referenced
    /// scrollview
    pub fn push_interrupt(&mut self, timestamp: Option<u64>) {
        //self.pan_log_x.clear();
        //self.pan_log_y.clear();
        //self.current_velocity = AxisVector { x: 0.0, y: 0.0, ..self.current_velocity };
        //self.x.signal_interrupt(timestamp.unwrap());
        self.y.signal_interrupt(timestamp.unwrap() as f64);
    }

    /// Set what device type is going to be providing any events that follow until the next source
    /// is declared
    pub fn set_source(&mut self, source: Source) {
        self.current_source = source;
    }
}
