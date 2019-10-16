//#include 
#include <memory>
#include <vector>

struct event {
    //
};

struct scrollviewstate {
    std::unique_ptr<std::vector<event>> past_events;
    scrollviewstate() {

    }
    ~scrollviewstate(void) {
        //
    }
};

//notes:
/**
 * if debug mode is set, we should aim to be user friendly
 * about hard failing if weird behavior is done,
 * like if a user tries to call get_pan without first
 * trying to snapshot during a frame
