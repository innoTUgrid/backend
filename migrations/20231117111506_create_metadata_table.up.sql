-- create the energy carrier lookup table

create table if not exists energy_carrier (
   id serial primary key,
   name text unique not null,
   created_at timestamptz not null default now(),
   updated_at timestamptz not null default now()
);
select trigger_updated_at('energy_carrier');

-- insert predefined energy carriers used by SMARD
insert into energy_carrier (name) values ('coal'),
                                  ('lignite'),
                                  ('oil'),
                                  ('gas'),
                                  ('nuclear'),
                                  ('solar'),
                                  ('hydro'),
                                  ('biomass'),
                                  ('biogas'),
                                  ('onwind'),
                                  ('offwind'),
                                  ('electricity'),
                                  ('pumped_storage'),
                                  ('other_renewable'),
                                  ('other_conventional');


-- create the timeseries metadata tables
create table if not exists meta (
    id serial primary key,
    identifier text collate "case_insensitive" not null,
    unit text collate "case_insensitive" not null,
    carrier integer references energy_carrier(id),
    consumption boolean,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique (identifier, unit)
);
select trigger_updated_at('meta');