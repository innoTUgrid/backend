-- remove index on carrier fk for emission factor table
drop index if exists idx_emission_factor_carrier;
