#include <caml/mlvalues.h>
#include "libtouch.h>

extern "C" {
    namespace ScrollView {
        CAMLPrim value rlscroll_create_scrollview() {
            return lscroll_create_scrollview();
        }

        CAMLPrim value rlscroll_destroy_scrollview(value handle) {
            lscroll_destroy_scrollview((lscroll_scrollview*)handle);
            return Val_unit;
        }
    }

    namespace Source {
        CAMLPrim value rlscroll_set_source_undefined(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_UNDEFINED);
            return Val_unit;
        }

        CAMLPrim value rlscroll_set_source_touchscreen(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_TOUCHSCREEN);
            return Val_unit;
        }

        CAMLPrim value rlscroll_set_source_touchpad(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_TOUCHPAD);
            return Val_unit;
        }

        CAMLPrim value rlscroll_set_source_mousewheel(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_MOUSEWHEEL);
            return Val_unit;
        }

        CAMLPrim value rlscroll_set_source_mousewheel_precise(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_MOUSEWHEEL_PRECISE);
            return Val_unit;
        }

        CAMLPrim value rlscroll_set_source_passthrough(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_PASSTHROUGH);
            return Val_unit;
        }

        CAMLPrim value rlscroll_set_source_passthrough_kinetic(value handle) {
            lscroll_set_input_source((lscroll_scrollview*)handle, LSCROLL_SOURCE_PASSTHROUGH_KINETIC);
            return Val_unit;
        }
    }

    namespace Events {
        CAMLPrim value rlscroll_signal_pan_x(value handle, value pan_x) {
            lscroll_add_scroll_x(
                    (lscroll_scrollview*)handle,
                    Int_val(pan_x)
            );
            return Val_unit;
        }

        CAMLPrim value rlscroll_signal_pan_y(value handle, value pan_y) {
            lscroll_add_scroll_y(
                    (lscroll_scrollview*)handle,
                    Int_val(pan_y)
            );
            return Val_unit;
        }

        CAMLPrim value rlscroll_signal_interrupt(value handle) {
            lscroll_add_scroll_interrupt((lscroll_scrollview*)handle);
            return Val_unit;
        }

        CAMLPrim value rlscroll_signal_release(value handle) {
            lscroll_add_scroll_release((lscroll_scrollview*)handle);
            return Val_unit;
        }
    }

    namespace Output {
        CAMLPrim value rlscroll_get_pan_x(value handle) {
            int64_t r = lscroll_get_pan_x((lscroll_scrollview*)handle);
            return Val_int(r);
        }

        CAMLPrim value rlscroll_get_pan_y(value handle) {
            int64_t r = lscroll_get_pan_y((lscroll_scrollview*)handle);
            return Val_int(r);
        }

        CAMLPrim value rlscroll_get_pos_x(value handle) {
            int64_t r = lscroll_get_pan_x((lscroll_scrollview*)handle);
            return Val_int(r);
        }

        CAMLPrim value rlscroll_get_pos_y(value handle) {
            int64_t r = lscroll_get_pan_y((lscroll_scrollview*)handle);
            return Val_int(r);
        }
    }

    namespace Manual {
        CAMLPrim value rlscroll_force_pan(value handle, value pan_x, value pan_y) {
            lscroll_force_pan(
                    (lscroll_scrollview*)handle,
                    Int_val(pan_x),
                    Int_val(pan_y)
            );
            return Val_unit;
        }

        CAMLPrim value rlscroll_force_jump(value handle, value pan_x, value pan_y) {
            lscroll_force_jump(
                    (lscroll_scrollview*)handle,
                    Int_val(pan_x),
                    Int_val(pan_y)
            );
            return Val_unit;
        }
    }
}
