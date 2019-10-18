#include <memory>
#include <ostream>
#include <vector>
#include <cstdlib>
#include <iostream>
#include <variant>
#include <mutex>
#include "libtouch.h"


namespace events {
    struct pan_event {
        //int64_t pan_x;
        //int64_t pan_y;
        int64_t pan_amount;
    };

    struct interrupt_event {
    };

    struct fling_event {
    };

    struct event {
        enum lscroll_input_source_t source;
        //std::variant<pan_event, fling_event, interrupt_event> content;
    };
}




struct lscroll_scrollview {
    std::recursive_mutex lock;

    struct {
        int64_t panned_by_x = 0;
        int64_t panned_by_y = 0;

        int64_t absolute_x = 0;
        int64_t absolute_y = 0;

        bool active = true; // TODO: change to false, only activate if pan ongoing (velocity is 0)
    } frame_pan;

    uint64_t content_width = 0;
    uint64_t content_height = 0;

    uint64_t viewport_width = 0;
    uint64_t viewport_height = 0;

    int64_t viewport_position_x = 0;
    int64_t viewport_position_y = 0;

    double current_velocity_x = 0;
    double current_velocity_y = 0;

    std::vector<std::variant<events::pan_event, events::interrupt_event, events::fling_event>> events_x;
    std::vector<std::variant<events::pan_event, events::interrupt_event, events::fling_event>> events_y;

    lscroll_scrollview() {

    }
    ~lscroll_scrollview(void) {

    }
};

extern "C" {

    struct lscroll_scrollview* lscroll_create_scrollview() {
        struct lscroll_scrollview* sv = new lscroll_scrollview;

        return sv;
    };

    void lscroll_destroy_scrollview(struct lscroll_scrollview* sv) {
        delete sv;
    }

    void lscroll_set_geometry(
            struct lscroll_scrollview* handle,
            uint64_t content_height,
            uint64_t content_width,
            uint64_t viewport_height,
            uint64_t viewport_width
    ) {
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif
        //insert values into header
        std::cout << "scrollview geometry updated to" << std::endl
            << "viewport w/h: " << viewport_width << ", " << viewport_height << std::endl
            << " content w/h: " << content_width << ", " << content_height << std::endl;

        // we may need to recompute position of viewport within content on resize, handle that here
        // also figure out on resize what a sane repositioning strategy is
        
#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    void lscroll_add_scroll_x(lscroll_scrollview* handle, int64_t motion_x) {
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif

        handle->events_x.push_back(events::pan_event{motion_x});

#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    void lscroll_add_scroll_y(lscroll_scrollview* handle, int64_t motion_y) {
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif

        handle->events_y.push_back(events::pan_event{motion_y});

#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    void lscroll_add_scroll(
            lscroll_scrollview* handle,
            int64_t motion_x,
            int64_t motion_y
    ) {
        // possibly flatten this out to avoid two locks
        // TODO: evaluate if worthwhile
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif

        handle->events_y.push_back(events::pan_event{motion_y});
        handle->events_x.push_back(events::pan_event{motion_x});

#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    void lscroll_add_scroll_interrupt(lscroll_scrollview* handle) {
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif

        handle->events_y.push_back(events::interrupt_event{});
        handle->events_x.push_back(events::interrupt_event{});

#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    void lscroll_add_scroll_release(lscroll_scrollview* handle) {
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif

        handle->events_y.push_back(events::fling_event{});
        handle->events_x.push_back(events::fling_event{});

#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    void lscroll_mark_frame(lscroll_scrollview* handle) {
#ifdef lscroll_thread_safe
        handle->lock.lock();
#endif
        // iterate through entries in event queues,
        // currently simply sum them and insert into
        // snapshot in handle

        // both relative, changed by events
        int64_t pan_x = 0;
        int64_t pan_y = 0;

        for(auto event: handle->events_x) {
            if(std::holds_alternative<events::pan_event>(event)) {
                pan_x += std::get<events::pan_event>(event).pan_amount;
            } else if(std::holds_alternative<events::fling_event>(event)) {
                //
            } else if(std::holds_alternative<events::interrupt_event>(event)) {
                //
            } else {
                std::cerr << "LibScroll: Not accounting for passed event in lscroll_mark_frame" << std::endl;
                exit(-1);
            }
        }

        for(auto event: handle->events_y) {
            if(std::holds_alternative<events::pan_event>(event)) {
                pan_y += std::get<events::pan_event>(event).pan_amount;
            } else if(std::holds_alternative<events::fling_event>(event)) {
                //
            } else if(std::holds_alternative<events::interrupt_event>(event)) {
                //
            } else {
                std::cerr << "LibScroll: Not accounting for passed event in lscroll_mark_frame" << std::endl;
                exit(-1);
            }
        }

        handle->frame_pan.panned_by_x = pan_x;
        handle->frame_pan.panned_by_y = pan_y;
        handle->frame_pan.absolute_x += pan_x;
        handle->frame_pan.absolute_y += pan_y;

        // TODO: need to constrain viewport to content

#ifdef lscroll_thread_safe
        handle->lock.unlock();
#endif
    }

    int64_t lscroll_get_pan_x(lscroll_scrollview* handle) {
        return handle->frame_pan.panned_by_x;
    }

    int64_t lscroll_get_pan_y(lscroll_scrollview* handle) {
        return handle->frame_pan.panned_by_y;
    }

    int64_t lscroll_get_pos_x(lscroll_scrollview* handle) {
        return handle->frame_pan.absolute_x;
    }

    int64_t lscroll_get_pos_y(lscroll_scrollview* handle) {
        return handle->frame_pan.absolute_y;
    }

    bool lscroll_query_pan_active(lscroll_scrollview* handle) {
        return handle->frame_pan.active;
    }

    void lscroll_force_pan(
            lscroll_scrollview* handle,
            int64_t x_dp,
            int64_t y_dp
    ) {
        lscroll_add_scroll(handle, x_dp, y_dp);
    }

    void lscroll_force_jump(
            lscroll_scrollview* handle,
            int64_t x_absolute,
            int64_t y_absolute
    ) {
        lscroll_add_scroll(
                handle,
                x_absolute - handle->frame_pan.absolute_x,
                y_absolute - handle->frame_pan.absolute_y);
    }
}



//notes:
/**
 * if debug mode is set, we should aim to be user friendly
 * about hard failing if weird behavior is done,
 * like if a user tries to call get_pan without first
 * trying to snapshot during a frame
 */
