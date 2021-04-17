table! {
    users (id) {
        id -> Bigint,
        discord_id -> Text,
        reddit_username -> Text,
        created ->Bigint,
    }
}
table! {
    events (id) {
        id -> Bigint,
        name -> Text,
        description -> Text,
        creator -> Text,
        active -> Bool,
        end ->Nullable<Bigint>,
        created ->Bigint,
    }
}