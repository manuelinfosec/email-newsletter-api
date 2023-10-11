-- Create Subscriptions Table
CREATE TABLE subscriptions (    -- create table named `subscriptions`
    id uuid NOT NULL,
    email TEXT NOT NULL UNIQUE, -- no restrictions on maximum length
    name TEXT NOT NULL, -- no restrictions on maximum length
    subscribed_at timestamptz NOT NULL, -- time-zone aware date and time type
    PRIMARY KEY (id)
);
