CREATE TABLE event (
    id INTEGER NOT NULL
        PRIMARY KEY,
    reactor_id INTEGER NOT NULL,
    irl_timestamp INTEGER NOT NULL,
    ingame_timestamp INTEGER NOT NULL
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

    status INTEGER NOT NULL
        CHECK (status IN (0, 1)),
    pretty_status TEXT AS (CASE
        WHEN status = 0 THEN 'inactive'
        WHEN status = 1 THEN 'active'
        ELSE 'unknown'
    END),

    temperature REAL NOT NULL,
    coolant_filled REAL NOT NULL,
    heated_coolant_filled REAL NOT NULL,
    fuel_filled REAL NOT NULL,
    waste_filled REAL NOT NULL,
    actual_burn_rate REAL NOT NULL,
    target_burn_rate REAL NOT NULL,
    damage_percent REAL NOT NULL,
    heating_rate REAL NOT NULL,
    boil_efficiency REAL NOT NULL
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
