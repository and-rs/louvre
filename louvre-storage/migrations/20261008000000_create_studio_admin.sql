CREATE TABLE studio_admin (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL
);
