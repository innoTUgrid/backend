-- Add up migration script here
alter table if exists meta add column if not exists microgrid_id integer default 0;