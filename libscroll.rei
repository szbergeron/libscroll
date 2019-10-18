module LibScroll = {
    // models libscroll as a flat api,
    // may change to be more structured later

    module ScrollView = {
        type t; // opaque per API
        external create: unit => t = "rlscroll_create_scrollview";
        external destroy: t => unit = "rlscroll_destroy_scrollview";
        // takes uints, TODO: pull in library to enforce unsigned args here to avoid spooky behavior
        external set_geometry: (t, int, int, int, int) => unit = "rlscroll_set_geometry";
    }

    module Source = {
        external set_source_undefined: ScrollView.t => unit = "rlscroll_set_source_undefined";
        external set_source_touchscreen: ScrollView.t => unit = "rlscroll_set_source_touchscreen";
        external set_source_touchpad: ScrollView.t => unit = "rlscroll_set_source_touchpad";
        external set_source_mousewheel: ScrollView.t => unit = "rlscroll_set_source_mousewheel";
        external set_source_mousewheel_precise: ScrollView.t => unit = "rlscroll_set_source_mousewheel_precise";
        external set_source_passthrough: ScrollView.t => unit = "rlscroll_set_source_passthrough";
        external set_source_passthrough_kinetic: ScrollView.t => unit = "rlscroll_set_source_passthrough_kinetic";
    }

    module Events = {
        external signal_pan_x: (ScrollView.t, int) => unit = "rlscroll_signal_pan_x";
        external signal_pan_y: (ScrollView.t, int) => unit = "rlscroll_signal_pan_y";
        external signal_release: (ScrollView.t) => unit = "rlscroll_signal_release";
        external signal_interrupt: (ScrollView.t) => unit = "rlscroll_signal_interrupt";
    }

    module Output = {
        external pan_x: (ScrollView.t) => int = "rlscroll_get_pan_x";
        external pan_y: (ScrollView.t) => int = "rlscroll_get_pan_y";
        external position_x: (ScrollView.t) => int = "rlscroll_get_pos_x";
        external position_y: (ScrollView.t) => int = "rlscroll_get_pos_y";
    }

    module Manual = {
        external pan: (ScrollView.t, int, int) => unit = "rlscroll_force_pan";
        external jump: (ScrollView.t, int, int) => unit = "rlscroll_force_jump";
    }
}

