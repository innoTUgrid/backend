-- Add up migration script here
create extension if not exists timescaledb cascade;
select create_hypertable('ts', 'series_timestamp');
