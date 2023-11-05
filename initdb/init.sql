CREATE TABLE IF NOT EXISTS test_table (
    id SERIAL PRIMARY KEY,
    Time TIMESTAMPTZ,
    "Production_of_CHP_kW" NUMERIC(12,6),
    "Total_Load_kW" NUMERIC(12,6),
    "Grid_Reference_SMARD_kW" NUMERIC(12,6),
    "Production_of_PV_kW" NUMERIC(12,6),
    "Brown Coal_MWh" NUMERIC(12,6),
    "Hard Coal_MWh" NUMERIC(12,6),
    "Other conventional fuel_MWh" NUMERIC(12,6),
    "Natural Gas_MWh" NUMERIC(12,6),
    "Biomass_MWh"	NUMERIC(12,6),
    "Hydropower_MWh" NUMERIC(12,6),
    "WindOffshore_MWh" NUMERIC(12,6),
    "WindOnshore_MWh"	NUMERIC(12,6),
    "Photovoltaic_MWh" NUMERIC(12,6),
    "Other Renewables_MWh" NUMERIC(12,6),
    "Pumped Storage_MWh" NUMERIC(12,6),
    "Trafo_out_1_power_Watts_15_min_mean" NUMERIC(12,6),
    "Trafo_out_2_power_Watts_15_min_mean" NUMERIC(12,6)
);

COPY test_table
FROM '/docker-entrypoint-initdb.d/inno2grid_all_data_cleaned_and_aligned.csv'
WITH (FORMAT csv, HEADER true, DELIMITER ',');