#include <cstdint>
/**
 * This library serves as an event interpretation library.
 * To use, you will need to take the raw events you recieve
 * on your platform and adapt it to a compatible input
 * interface. You are expected to call get_pan() once on every frame.
 * It expects an estimation of the next frametime as well as how
 * long until the current frame will be rendered. This allows overshoot
 * calculation to take place.
 */

#define USE_SCALE_FACTOR true

struct pan_transform {
    int64_t x; // x axis pan amount in dp
    int64_t y; // y axis pan amount in dp
    bool panned : 1; // only true if a pan event has occurred.
                 // A transform can be skipped if this is false, otherwise assume
                 // that a pan has occurred and do a transform of the viewport by
                 // x and y
                 //
                 // also indicates that no further pan or state change
                 // will occur without adding another event to
                 // the queue, so any render loop can block safely

    double velocity_x; // gives current x axis velocity in dp, can be used for overscroll behavior
    double velocity_y; // gives y axis velocity
};

/**
 * Gets a pan event detailing how to transform
 * the current viewport
 */
struct pan_transform get_pan();

/**
 * Sets how long the average frame is as well as how far
 * in the future to predict a pan. This allows us to slightly
 * overshoot any pan to minimize percieved lag
 */
void set_predict(float ms_to_vsync, float ms_avg_frametime);

/**
 * Shorthand for calling set_predict followed by get_pan
 * Use this if frametimes or render latency
 * are highly variable to minimize jank or stutter.
 */
struct pan_transform get_pan_predict(float ms_to_vsync, float ms_avg_frametime);

int64_t get_pan_x(); // gets x component of current pan. WARN: mutates internal state: clears x axis event buffer
int64_t get_pan_y(); // gets y component of current pan. WARN: mutates internal state: clears y axis event buffer

/**
 * set_input_source should be properly used always, since
 * if input is assumed to be a touchpad and turns out to
 * be a touchscreen, an acceleration curve will be applied
 * which will desynchronize touch point and panning
 *
 * scroll_natural also only applies to 
 */

void set_scale_factor(float x_factor, float y_factor); // normalization factor for quirky devices

enum input_source_t {
    UNDEFINED, // acts identically to PASSTHROUGH_KINETIC,
               // only use when no hint is available as
               // to what input source is
    TOUCHSCREEN,
    TOUCHPAD,
    MOUSEWHEEL,
    MOUSEWHEEL_PRECISE,
    PASSTHROUGH, // use for inputs that have their own drivers
                 // handling any acceleration curves
                 // or overshoot; disables any
                 // input processing here, only sum pan distance
                 // Examples: TrackPoint, trackball, mousekeys
    PASSTHROUGH_KINETIC, // use as prior, but keep
                         // kinetic scrolling after scroll_release event
};

/**
 * can be called at any time between calls to get_pan_*
 * and is indipotent
 *
 * output of any get_pan_* call is interpreted
 * through the lens of the most recently set source
 */
void set_input_source(enum input_source_t input_source);

int add_scroll(
    int64_t motion_x, // x axis motion reported by device
    int64_t motion_y // y axis motion reported by device
);

/**
 * similar to add_scroll(motion_x, motion_y),
 * use for when input device delivers axes
 * as separate events
 */
int add_scroll_x(int64_t motion_x);
int add_scroll_y(int64_t motion_y);

/**
 * analogous to "was scrolling kinetically,
 * until user put two fingers on touchpad"
 */
void add_scroll_interrupt();

/**
 * triggers kinetic scrolling,
 * last event to be sent during
 * a "flick" action
 */
void add_scroll_release();
