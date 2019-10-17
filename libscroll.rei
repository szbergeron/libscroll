module LibScroll = {
    // models libscroll as a flat api,
    // may change to be more structured later

    module ScrollView = {
        type t; // opaque per API
        external create: unit => t = "rlscroll_create_scrollview";
        external destroy: t => unit = "rlscroll_destroy_scrollview";
    }
}

