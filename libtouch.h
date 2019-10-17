#include <stdint.h>
#include <stdbool.h>
/**
 * This library serves as an event interpretation library.
 * To use, you will need to take the raw events you recieve
 * on your platform and adapt it to a compatible input
 * interface. You are expected to call get_pan() once on every frame.
 * It expects an estimation of the next frametime as well as how
 * long until the current frame will be rendered. This allows overshoot
 * calculation to take place.
 */

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

//NOTE: this library is not yet multithreading safe, but that is TODO

/**
 * Any of these options can be logical or'ed together
 * and passed to set_options
 */
#ifdef __cplusplus
extern "C" {
#endif

    enum lscroll_options {
        LIBSCROLL_IMPRECISE_SCROLLS_SMOOTHLY = 0x1, // controls whether large jumps from imprecise devices (keyboard, clickwheel) should animate smoothly
    };

    /**
     * Do initialization tasks for and return a handle to a new scrollview
     *
     * Default geometry will be used for this variant, and can be updated
     * with signal_geometry() on a modified scrollview
     */
    struct lscroll_scrollview* lscroll_create_scrollview();

    ///**
    // * Create a new scrollview and simultaneously default
    // * initialize the geometry of the scrollview to that
    // * of the passed scrollview
    // */
    //struct scrollview* create_scrollview(struct scrollview view);

    /**
     * Tears down and frees the referenced scrollview
     *
     * The handle passed here should be considered invalid
     * after this function has been called
     */
    void lscroll_destroy_scrollview(struct lscroll_scrollview* handle);

    /** Set the geometry for the current scrollview */
    void lscroll_set_geometry(
            struct lscroll_scrollview* handle,
            uint64_t content_height,
            uint64_t content_width,
            uint64_t viewport_height,
            uint64_t viewport_width
    );

    /**
     * Allows forcing a relative scroll by x, y dp in the current scrollview
     *
     * Example use case: user uses a keyboard shortcut to jump down by a page
     */
    void lscroll_force_pan(
            struct lscroll_scrollview* handle,
            int64_t x_dp,
            int64_t y_dp
    );

    /**
     * Allows forcing a scroll to position x, y dp in the current scrollview
     *
     * Example use case: user jumps to an absolute line number in a text editor
     */
    void lscroll_force_jump(
            struct lscroll_scrollview* handle,
            int64_t x_dp,
            int64_t y_dp
    );

    /**
     * Sets how long the average frame is as well as how far
     * in the future to predict a pan. This allows us to slightly
     * overshoot any pan to minimize percieved lag
     */
    void lscroll_set_predict(
            struct lscroll_scrollview* handle,
            float ms_to_vsync,
            float ms_avg_frametime
    );

    /**
     * WARN: the following get_[...]() and query_[...] functions should only be called after a call to mark_frame()
     */

    /**
     * Returns true if a pan is ongoing and rendering should continue
     * (render thread should not block)
     *
     * A scrollview may be active still even if pan amount is zero,
     * for instance if a scrollview is briefly balanced on a
     * magnetic boundary edge
     */
    bool lscroll_query_pan_active(struct lscroll_scrollview* handle);

    /** gets x component of current pan. WARN: mutates internal state: clears x axis event buffer */
    int64_t lscroll_get_pan_x(struct lscroll_scrollview* handle);

    /** gets y component of current pan. WARN: mutates internal state: clears y axis event buffer */
    int64_t lscroll_get_pan_y();

    /** gets absolute x position of current viewport into/relative to content */
    int64_t lscroll_get_pos_x();

    /** gets absolute y position of current viewport into/relative to content */
    int64_t lscroll_get_pos_y();

    ///**
    // * set_input_source should be properly used always, since
    // * if input is assumed to be a touchpad and turns out to
    // * be a touchscreen, an acceleration curve will be applied
    // * which will desynchronize touch point and panning
    // *
    // * scroll_natural also only applies to 
    // */
    //void set_scale_factor(float x_factor, float y_factor); // normalization factor for quirky devices

    enum lscroll_input_source_t {
        LSCROLL_SOURCE_UNDEFINED, // acts identically to PASSTHROUGH_KINETIC,
                   // only use when no hint is available as
                   // to what input source is
        LSCROLL_SOURCE_TOUCHSCREEN,
        LSCROLL_SOURCE_TOUCHPAD,
        LSCROLL_SOURCE_MOUSEWHEEL,
        LSCROLL_SOURCE_MOUSEWHEEL_PRECISE,
        LSCROLL_SOURCE_PASSTHROUGH, // use for inputs that have their own drivers
                     // handling any acceleration curves
                     // or overshoot; disables any
                     // input processing here, only sum pan distance
                     // Examples: TrackPoint, trackball, mousekeys
        LSCROLL_SOURCE_PASSTHROUGH_KINETIC, // use as prior, but keep
                             // kinetic scrolling after scroll_release event
    };

    /**
     * should be called before any add_scroll_[...]() function call for a given device,
     * as any scroll event call is interpreted as coming from the last input source set
     */
    void lscroll_set_input_source(enum lscroll_input_source_t input_source);

    /** Add some pan event to the scrollview referenced by handle */
    void lscroll_add_scroll(
            struct lscroll_scrollview* handle,
            int64_t motion_x, // x axis motion reported by device
            int64_t motion_y // y axis motion reported by device
    );

    /**
     * similar to add_scroll(motion_x, motion_y),
     * use for when input device delivers axes
     * as separate events
     */
    void lscroll_add_scroll_x(struct lscroll_scrollview* handle, int64_t motion_x);
    void lscroll_add_scroll_y(struct lscroll_scrollview* handle, int64_t motion_y);

    /**
     * analogous to "was scrolling kinetically,
     * until user put two fingers back on touchpad"
     */
    void lscroll_add_scroll_interrupt(struct lscroll_scrollview* handle);

    /**
     * triggers kinetic scrolling,
     * last event to be sent during
     * a "flick" action
     */
    void lscroll_add_scroll_fling(struct lscroll_scrollview* handle);

    /**
     * Call this as late in the rendering pipeline as possible before asking
     * for current pan/geometry.
     *
     * Internally this takes a snapshot of the proposed pan amount
     * and locks those numbers until the next call to mark_frame()
     */
    void lscroll_mark_frame(struct lscroll_scrollview* handle);

#ifdef __cplusplus
}
#endif
