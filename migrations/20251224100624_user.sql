-- Add migration script here
create table if not exists user (
    id INTEGER
    , lastfm_name TEXT
    , lastfm_key TEXT
    , auth_at INTEGER
    , lastfm_subscriber INTEGER
    , primary key (id)
);
