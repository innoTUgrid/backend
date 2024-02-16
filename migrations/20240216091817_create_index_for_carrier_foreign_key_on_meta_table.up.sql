-- create a b tree index for the carrier foreign key on the meta table
create index if not exists idx_meta_carrier on meta(carrier);
