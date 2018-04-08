table! {
    audit_log (id) {
        id -> Int8,
        user_id -> Int8,
        index -> Text,
        query -> Text,
        created_at -> Timestamptz,
    }
}

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

joinable!(audit_log -> users (user_id));

allow_tables_to_appear_in_same_query!(
    audit_log,
    sessions,
    users,
);
