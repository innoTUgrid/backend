alter table config alter created_at type timestamptz using created_at at time zone 'UTC';
alter table config alter updated_at type timestamptz using updated_at at time zone 'UTC';