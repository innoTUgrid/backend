create table config (
    id serial primary key,
    config jsonb not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
);
