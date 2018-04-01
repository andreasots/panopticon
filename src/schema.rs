table! {
    sessions (id) {
        id -> Text,
        data -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int8,
        name -> Text,
        password -> Bytea,
        groups -> Array<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
