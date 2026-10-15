CREATE TABLE social_identities (
    account_id TEXT PRIMARY KEY NOT NULL,
    token TEXT NOT NULL CHECK(length(token) = 64 AND token NOT GLOB '*[^0-9a-fA-F]*')
);
