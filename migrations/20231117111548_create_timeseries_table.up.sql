create table if not exists ts (
                                  id bigserial,
                                  series_timestamp timestamptz not null,
                                  series_value double precision not null,
                                  meta_id integer,
                                  created_at timestamptz not null default now(),
                                  updated_at timestamptz not null default now(),
                                  primary key(id, series_timestamp),
                                  foreign key (meta_id) references meta(id)
);
select trigger_updated_at('ts');
