#include <memory>
#include <ostream>
#include <vector>
#include <cstdlib>
#include <iostream>
#include <variant>
#include "libtouch.h"


namespace events {
    struct pan_event {
        int64_t pan_x;
        int64_t pan_y;
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
    uint64_t content_height = 0;
    uint64_t content_width = 0;

    uint64_t viewport_height = 0;
    uint64_t viewport_width = 0;

    std::vector<events::pan_event> past_events;

    lscroll_scrollview() {

    }
    ~lscroll_scrollview(void) {

    }
};

extern "C" {

struct lscroll_scrollview* lscroll_create_scrollview() {
    struct lscroll_scrollview* sv = new struct lscroll_scrollview;

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
    //insert values into header
    std::cout << "scrollview geometry updated to" << std::endl
        << "viewport w/h: " << viewport_width << ", " << viewport_height << std::endl
        << " content w/h: " << content_width << ", " << content_height << std::endl;
}

/*
void lscroll_signal_geometry(struct lscroll_scrollview* sv) {
    std::cout << "scrollview geometry updated to" << std::endl
        << "viewport w/h: " << sv->viewport_width << ", " << sv->viewport_height << std::endl
        << " content w/h: " << sv->content_width << ", " << sv->content_height << std::endl;

    sv->state->content_height = sv->content_height;
    sv->state->content_width = sv->content_width;
    sv->state->viewport_height = sv->viewport_height;
    sv->state->viewport_width = sv->viewport_width;
}
*/
}



//notes:
/**
 * if debug mode is set, we should aim to be user friendly
 * about hard failing if weird behavior is done,
 * like if a user tries to call get_pan without first
 * trying to snapshot during a frame
 */
