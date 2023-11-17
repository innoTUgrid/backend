-- initial migration
-- setup primary tables and define trigger to compute created and updated fields which every table should have
-- ensure every table has created_at and updated_at columns
create or replace function set_updated_at()
    returns trigger as
    $$
    begin
        new.updated_at = now();
        return NEW;
    end;
    $$ language plpgsql;

create or replace function trigger_updated_at(tablename regclass)
    returns void as
    $$
    begin execute format(
            'CREATE TRIGGER set_updated_at
            BEFORE UPDATE on %s
            FOR EACH ROW WHEN
            (OLD is distinct from NEW)
            EXECUTE FUNCTION set_updated_at();', tablename);
    end;
    $$ language plpgsql;

create collation case_insensitive (provider = icu, locale = 'und-u-ks-level2', deterministic = false);