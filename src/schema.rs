// @generated automatically by Diesel CLI.

diesel::table! {
    adxl_datas (id) {
        id -> Int4,
        device_id -> Int4,
        ts -> Timestamptz,
        x -> Float4,
        y -> Float4,
        z -> Float4,
        xf -> Float4,
        yf -> Float4,
        zf -> Float4,
        ax -> Float4,
        ay -> Float4,
        az -> Float4,
        axf -> Float4,
        ayf -> Float4,
        azf -> Float4,
        t -> Float4,
        tf -> Float4,
        bat -> Float4,
    }
}

diesel::table! {
    humiture_datas (id) {
        id -> Int4,
        #[max_length = 255]
        sn -> Varchar,
        #[max_length = 255]
        device_id -> Varchar,
        group_id -> Int4,
        type_id -> Int4,
        ts -> Timestamp,
        temperature -> Float4,
        humidity -> Float4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    adxl_datas,
    humiture_datas,
);
