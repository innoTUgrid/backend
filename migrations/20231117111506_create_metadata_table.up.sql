-- Add up migration script here
create table if not exists meta (
    id serial primary key,
    identifier text collate "case_insensitive" not null,
    unit text collate "case_insensitive" not null,
    carrier text,
    consumption boolean,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique (identifier, unit)
);
select trigger_updated_at('meta');