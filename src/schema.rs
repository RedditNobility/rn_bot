table! {
    users (uid) {
        uid -> Bigint,
        discord_id -> Text,
        reddit_username -> Text,
        created ->Bigint,
    }
}
table! {
    events (eid) {
        eid -> Bigint,
        name -> Text,
        description -> Text,
        creator -> Text,
        active -> Bool,
        end ->Nullable<Bigint>,
        created ->Bigint,
    }
}