-- Add migration script here
ALTER TABLE youtuber_permissions
ADD COLUMN custom_url TEXT;
