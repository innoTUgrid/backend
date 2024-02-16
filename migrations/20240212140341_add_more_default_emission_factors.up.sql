-- other_conventional as average of conventional sources
insert into emission_factor
(carrier, factor, unit, source, source_url)
select id,
       (select avg(emission_factor.factor)
            from emission_factor
                     join energy_carrier on emission_factor.carrier = energy_carrier.id
            where energy_carrier.name in ('gas', 'lignite', 'coal', 'nuclear')
        )
     , 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'other_conventional';

-- other renewables as average of all renewables
insert into emission_factor
(carrier, factor, unit, source, source_url)
select id,
       (select avg(emission_factor.factor)
        from emission_factor
                 join energy_carrier on emission_factor.carrier = energy_carrier.id
        where energy_carrier.name in ('solar', 'hydro', 'biomass', 'biogas', 'onwind', 'offwind')
       )
        , 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'other_renewable';

insert into emission_factor
(carrier, factor, unit, source, source_url)
select id,
       (select avg(emission_factor.factor)
        from emission_factor
                 join energy_carrier on emission_factor.carrier = energy_carrier.id
        where energy_carrier.name in ('solar', 'hydro', 'biomass', 'biogas', 'onwind', 'offwind')
       )
        , 'kgco2eq/kwh', 'IPCC', null
from energy_carrier where energy_carrier.name = 'pumped_storage';
