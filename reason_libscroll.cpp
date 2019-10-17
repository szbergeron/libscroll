#include <caml/mlvalues.h>
#include "libtouch.h>

extern "C" {
    CAMLPrim value rlscroll_create_scrollview() {
        return lscroll_create_scrollview();
    }

    CAMLPrim value rlscroll_destroy_scrollview(value handle) {
        lscroll_destroy_scrollview((lscroll_scrollview*)handle);
        return Val_unit;
    }
}
