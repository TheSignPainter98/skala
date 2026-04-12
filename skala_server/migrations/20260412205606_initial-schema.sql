CREATE TABLE reactor (
    id INTEGER NOT NULL
        PRIMARY KEY,
    name TEXT NOT NULL
        UNIQUE
) STRICT;

CREATE INDEX reactor__name_index ON reactor(name);

CREATE TABLE advice (
    id INTEGER NOT NULL
        PRIMARY KEY,
    reactor_id INTEGER NOT NULL
        REFERENCES reactor(id),

    status INTEGER NOT NULL
        DEFAULT 0
        CHECK (status IN (0, 1, 2)),
    pretty_status TEXT AS (CASE
        WHEN status = 0 THEN 'received'
        WHEN status = 1 THEN 'advised'
        WHEN status = 2 THEN 'failed'
        ELSE 'unknown'
    END),

    advice TEXT NULL
        DEFAULT NULL,

    reactor_status INTEGER NOT NULL
        CHECK (reactor_status IN (0, 1)),
    pretty_reactor_status TEXT AS (CASE
        WHEN reactor_status = 0 THEN 'inactive'
        WHEN reactor_status = 1 THEN 'active'
        ELSE 'unknown'
    END),

    reactor_temperature REAL NOT NULL,
    reactor_coolant_filled REAL NOT NULL,
    reactor_heated_coolant_filled REAL NOT NULL,
    reactor_fuel_filled REAL NOT NULL,
    reactor_waste_filled REAL NOT NULL,
    reactor_actual_burn_rate REAL NOT NULL,
    reactor_target_burn_rate REAL NOT NULL,
    reactor_damage_percent REAL NOT NULL,
    reactor_heating_rate REAL NOT NULL,
    reactor_boil_efficiency REAL NOT NULL,

    CHECK ((status = 1) = (advice IS NOT NULL))
) STRICT;
