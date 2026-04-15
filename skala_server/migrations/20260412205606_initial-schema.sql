CREATE TABLE event (
    id INTEGER NOT NULL
        PRIMARY KEY,
    reactor_id INTEGER NOT NULL,
    irl_timestamp INTEGER NOT NULL,
    ingame_timestamp TEXT NOT NULL
) STRICT;

CREATE TABLE reactor (
    id INTEGER NOT NULL
        PRIMARY KEY,
    name TEXT NOT NULL
        UNIQUE
) STRICT;

CREATE INDEX reactor__name_index ON reactor(name);

CREATE TABLE reactor_state (
    id INTEGER NOT NULL
        PRIMARY KEY,
    event_id INTEGER NOT NULL
        REFERENCES event(id),

    intact INTEGER NOT NULL
        CHECK (intact IN (0, 1)),
    pretty_intact TEXT AS (CASE
        WHEN intact = 0 THEN 'destroyed'
        WHEN intact = 0 THEN 'intact'
    END),

    mode INTEGER NULL
        DEFAULT NULL
        CHECK (mode IS NULL OR mode IN (0, 1)),
    pretty_mode TEXT AS (CASE
        WHEN mode = 0 THEN 'inactive'
        WHEN mode = 1 THEN 'active'
        WHEN NULL THEN '-'
        ELSE 'unknown'
    END),

    temperature REAL NULL
        DEFAULT NULL,
    coolant_filled REAL NULL
        DEFAULT NULL,
    heated_coolant_filled REAL NULL
        DEFAULT NULL,
    fuel_filled REAL NULL
        DEFAULT NULL,
    waste_filled REAL NULL
        DEFAULT NULL,
    actual_burn_rate REAL NULL
        DEFAULT NULL,
    target_burn_rate REAL NULL
        DEFAULT NULL,
    damage_percent REAL NULL
        DEFAULT NULL,
    heating_rate REAL NULL
        DEFAULT NULL,
    boil_efficiency REAL NULL
        DEFAULT NULL
) STRICT;

CREATE TABLE advice (
    id INTEGER NOT NULL
        PRIMARY KEY,
    event_id INTEGER NOT NULL
        REFERENCES event(id),

    action INTEGER NOT NULL
        CHECK (action IN (0, 1)),
    pretty_action TEXT AS (CASE
        WHEN action = 0 THEN 'no-action'
        WHEN action = 1 THEN 'scram'
        ELSE 'unknown'
    END),

    reasoning TEXT NOT NULL
) STRICT;
