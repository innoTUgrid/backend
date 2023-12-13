-- remove the emission factor table
create table if not exists emission_factor (
   id serial primary key,
   carrier integer references energy_carrier(id) not null,
   factor double precision not null,
   unit text not null,
   source text not null,
   source_url text,
   created_at timestamptz not null default now(),
   updated_at timestamptz not null default now()
);
select trigger_updated_at('emission_factor');

