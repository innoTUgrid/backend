-- create a b tree index on the carrier foreign key on the emission factor table
create index if not exists idx_emission_factor_carrier on emission_factor(carrier);
