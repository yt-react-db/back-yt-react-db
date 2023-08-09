-- Add migration script here
CREATE TYPE Permissions AS ENUM (
    'yes',
    'yes_with_delay',
    'no'
);

CREATE TABLE youtuber_permissions (
    id SERIAL PRIMARY KEY,
    channel_id CHAR(24) NOT NULL,
    channel_title TEXT NOT NULL,

    -- permission 1
    can_react_live Permissions NOT NULL,
    live_reaction_delay VARCHAR(4), -- "0" up to "999y", "999h", "999d", "999w", "999m",
        -- h: hours, d: days, w: weeks, y: years
    100h
    100d
    100w
    000m
    100y

    -- permission 2
    can_upload_reaction Permissions NOT NULL
);