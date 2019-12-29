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

//use std::ops;


/**
 * Example usage:
 * // TODO update usage example
 * 1. Create some `struct scrollview` locally and pass geometry
 *      and expected behavior as specified in struct
 *
 * 2. Pass said struct by value to create_scrollview(), storing
 *      the returned scrollview handle for future use
 *      in conjunction with the associated UI scrollview
 *
 * 3. Use set_predict() with estimations of average frametimes
 *      and how far into a frame period each get_pos/get_pan call
 *      will occur
 *
 * 4. In event loop, recieve and pass any scroll events through
 *      add_scroll(), add_scroll_interrupt(), add_scroll_release()
 *      and related event signaling functions. Strict ordering
 *      or summation are not required here, just pass info as
 *      it comes in from the device
 *
 * 5. On each render loop iteration, call mark_frame() and then use get_pan_[x/y]() or
 *      get_pos_[x/y]() to find where to transform the content to
 *      under the viewport, no intermediate processing required
 *
 * 6. Call destroy_scrollview(), passing the scrollview handle
 *      from earlier to clean up scrollview on exit
 */

#[derive(Default)]
pub struct Scrollview {
    content_height: u64,
    content_width: u64,
    viewport_height: u64,
    viewport_width: u64,

    current_velocity: AxisVector<f64>,
    current_position: AxisVector<f64>,

    prior_position: AxisVector<f64>,

    //event_queue: crate::circular_backqueue::ForgetfulLogQueue<Event>,
    event_queue: circular_backqueue::ForgetfulLogQueue<AxisVector<f64>>,
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

    fn update(&mut self, axis: Axis, magnitude: T) {
        match axis {
            Axis::Horizontal => self.x = magnitude,
            Axis::Vertical => self.y = magnitude,
        }
    }

    fn append(&mut self, axis: Axis, magnitude: T) {
        match axis {
            Axis::Horizontal => self.x = magnitude + self.x,
            Axis::Vertical => self.y = magnitude + self.y,
        }
    }
}

//impl<T: num::Float> AxisVector<T> where T: std::convert::From<f64>, f64: std::convert::From<T> {
impl AxisVector<f64> {
    fn decay_active(&self) -> bool {
        self.decaying && self.x > self.x_threshold && self.y > self.y_threshold
    }

    fn decay_start(&mut self) {
        self.decaying = true;
    }

    fn step_frame(&mut self) {
        if self.decay_active() {
            self.x = fling_decay(self.x);
            self.y = fling_decay(self.y);
        }

        if self.x < self.x_threshold && self.y < self.y_threshold {
            self.decaying = false;
        }
    }
}

// TODO: change to pythagorean subtract
/*impl<T> ops::Sub<AxisVector<T>> for AxisVector<T> where T: num::Num, T: PartialOrd {
    type Output = AxisVector<T>;

    fn sub(self, rhs: AxisVector<T>) -> AxisVector<T> {
        AxisVector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            ..self

        }
    }
}*/

#[derive(Copy)]
#[derive(Clone)]
pub enum Axis {
    Horizontal,
    Vertical,
}

//#[derive(Default)]
pub enum Event {
    Pan { axis: Axis, amount: i32 }, // doesn't use AxisVector since some platforms only send one pan axis at once // TODO: consider AxisVector[Optional]
    Fling,
    Interrupt,
    //Zoom?
}

// pub interface
impl Scrollview {
    /// Create a new scrollview with default settings
    ///
    /// Warning: these settings are unlikely to be
    /// particularly useful, so set_geometry(), set_avg_frametime(), and any
    /// other relevant initialization functions still need to be used
    pub fn new() -> Scrollview {
        /*Scrollview {
            prior_position: Default::default(),
            current_position: Default::default(),
            current_velocity: Default::default(),
            event_queue: circular_backqueue::ForgetfulLogQueue::new(10),
            content_height: 0,
            content_width: 0,
            viewport_height: 0,
            viewport_width: 0,
        }*/
        Default::default()
    }

    /// Deletes/deinitializes the current scrollview
    ///
    /// Primarily intended for ffi use, Scrollview implements Drop
    /// where deinitialization is required, so this is only useful
    /// for ffi use
    pub fn del(_: Scrollview) {}

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
    pub fn push_event(
        &mut self,
        event: &Event
    ) {
        match event {
            Event::Pan { axis, amount } => self.push_pan(*axis, *amount),
            Event::Fling => self.push_fling(),
            Event::Interrupt => self.push_interrupt(),
        }
    }

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
    pub fn step_frame(&mut self) {
        self.current_velocity.step_frame();
    }
    
    /// Should be called at scrollview initialization time.
    /// Will internally flush any active events or animations,
    /// so should only be used when scrollview is inactive or
    /// when absolutely necessary (monitor refresh rate changes)
    ///
    /// Used for position prediction (better pan tracking)
    pub fn set_avg_frametime(&mut self, milliseconds: f64) {
        //
    }

    /// Indicate how long there is until the next frame will be rendered
    /// to the screen.
    ///
    /// Used for position prediction (better pan tracking)
    pub fn set_next_frame_predict(&mut self, milliseconds: f64) {
        //
    }

    /// Get the position of the content's top-left corner relative to
    /// the top-left corner of the viewport
    ///
    /// NOTE: either axis may be negative. This indicates an overscroll is occurring.
    /// Recommended way of handling this is to checkerboard that area visually
    /// and draw true to the provided geometry. This matches platform behavior for OSX and Windows,
    /// as well as some Linux programs, and is often called the "rubber band effect"
    pub fn get_position_absolute(&self) -> AxisVector<f64> {
        self.current_position
    }

    /// Get the position of the content's top-left corner relative to
    /// its position before the most recent step_frame(), saying how much
    /// the content should be moved from its last position
    pub fn get_position_relative(&self) -> AxisVector<f64> {
        self.current_position.difference(self.prior_position)
    }
}

// private impl
impl Scrollview {
    fn push_pan(&mut self, axis: Axis, amount: i32) {
        self.current_velocity.update(axis, f64::from(amount));
        self.current_position.append(axis, f64::from(amount));
    }

    fn push_fling(&mut self) {
        self.current_velocity.decay_start();
    }

    fn push_interrupt(&mut self) {
        //
    }
}

// should be changed later to allow different curves, 
fn fling_decay(from: f64) -> f64 {
    //f64::from(from)
    //T::from(from.into().powf(1.32)).unwrap()
    from.powf(0.998)
    //T::from(f64::from(from).powf(1.32))
    //from.into::<f64>().powf(1.32).into::<T>()
}

impl circular_backqueue::ForgetfulLogQueue<AxisVector<f64>> {
    pub fn get_or_avg(&self, position: usize) -> AxisVector<f64> {
        let ret = self.get(position);

        match ret {
            Some(av) => av.clone(),
            None => {
                let mut sum_av: AxisVector<f64> = Default::default();
                for av in self.all() {
                    sum_av.x += av.x;
                    sum_av.y += av.y;
                }

                sum_av.x /= self.size() as f64;
                sum_av.y /= self.size() as f64;

                sum_av
            }
        }
    }
}

/*
 * Impl notes
 *
 * Bounce:
 *
 * Fling:
 *
 * Accel:
 */
