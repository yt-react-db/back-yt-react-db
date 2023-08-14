-- Add migration script here
CREATE TYPE Permission AS ENUM (
    'yes',
    'yes_with_delay',
    'no'
);

CREATE TABLE youtuber_permissions (

    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    channel_id CHAR(24) NOT NULL UNIQUE,
    channel_title TEXT NOT NULL,

    -- permission 1
    can_react_live Permission NOT NULL,
    live_reaction_delay VARCHAR(4) DEFAULT NULL, -- "0" up to "999y", "999h", "999d", "999w", "999m",
        -- h: hours, d: days, w: weeks, y: years

    -- permission 2
    can_upload_reaction Permission NOT null,
    upload_reaction_delay VARCHAR(4) DEFAULT NULL, -- same as live_reaction_delay

    last_updated_at TIMESTAMPTZ DEFAULT NOW()

);

CREATE INDEX youtuber_permissions__channel_id__idx
ON youtuber_permissions (channel_id);

CREATE INDEX youtuber_permissions__channel_title__idx
ON youtuber_permissions (channel_title);


-- It's important to hold an history so that youtubers can't deny that they allowed
-- people to react to their videos before and take down videos or claim revenue!
--
-- here we hold the previous values
CREATE TABLE youtuber_permissions_history (

    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    channel_id CHAR(24) NOT NULL,

    can_react_live Permission NOT NULL,
    live_reaction_delay VARCHAR(4) DEFAULT NULL,

    can_upload_reaction Permission NOT NULL,
    upload_reaction_delay VARCHAR(4) DEFAULT NULL,

    -- date when it was originally updated (not now)
    set_at TIMESTAMPTZ
);

CREATE OR REPLACE FUNCTION copy_before_update()
RETURNS TRIGGER AS $$
BEGIN

    INSERT INTO youtuber_permissions_history(
        channel_id,
        can_react_live, live_reaction_delay,
        can_upload_reaction, upload_reaction_delay,
        set_at
    )
    VALUES (
        OLD.channel_id,
        OLD.can_react_live, OLD.live_reaction_delay,
        OLD.can_upload_reaction, OLD.upload_reaction_delay,
        OLD.last_updated_at
    );

    RETURN NEW; -- Return the original row

END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_before_update_youtuber_permissions
BEFORE UPDATE ON youtuber_permissions
FOR EACH ROW
EXECUTE FUNCTION copy_before_update();
