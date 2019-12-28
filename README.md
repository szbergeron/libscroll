# libscroll [WIP]
A drop-in solution to add smooth, responsive, scrolling to any view.
Takes in individual input events, outputs a simple pan amount.

Main library interface is implemented in src/lib.rs

# Usage:
1. Allocate a scrollview with Scrollview::new(), the returned scrollview is the object to which all following directions are applied (through method calls)
2. Use set\_geometry with the size of both the content of the scrollview and the size of the viewport at the current time
3. Use set\_avg\_frametime based on current FPS or some other metric to allow position prediction. If this information isn't available, a safe default is 0ms, but this will introduce additional perceptible lag
4. If render and event loop are separate, split them here. Place the scrollview in an Arc<Mutex<>> to ensure atomic access.
    Note: scrollviews may be made atomic and internally mutable in the future. This change should not alter backwards compatibility, but should improve usability for this use case

Within event loop:

1. Take any outstanding events from platform driver/provider (SDL, Libinput, Wayland event provider) and use push\_event() to add them to the internal queue.
2. Loop back to 5

Within render loop:

1. Call set\_next\_frame\_predict() to set approximately how long it will be from now until content is rendered to screen, or 0 if unsure (at cost of additional latency)
2. Call step\_frame() to both account for any newly emplaced events, and to advance any ongoing animations by one tick
3. If animating() is true, use either get\_position\_absolute() or get\_position\_relative() to see where to move the viewport, or by how much. These calls are idempotent and non-mutating. Call them whenever is convenient after step\_frame()

That's it! Everything else is handled behind the scenes
