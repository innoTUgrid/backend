-- create an index for the meta id foreign key on the ts table
create index if not exists idx_ts_meta_id on ts(meta_id);
