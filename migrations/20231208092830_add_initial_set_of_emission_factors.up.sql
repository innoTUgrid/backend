-- insert some basic emission factors into the emission factor table
-- these factors are median values taken from https://en.wikipedia.org/wiki/Life-cycle_greenhouse_gas_emissions_of_energy_sources

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.82, 'kgco2eq/kwh' ,'IPCC', null
from energy_carrier where energy_carrier.name = 'lignite';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.49, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'gas';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.82, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'coal';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.024, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'hydro';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.048, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'solar';


insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.012, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'nuclear';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.011, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'onwind';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.012, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'offwind';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.038, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'geothermal';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id, 0.230, 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'biomass';
