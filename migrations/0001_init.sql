CREATE TABLE IF NOT EXISTS users (
    login       TEXT PRIMARY KEY,
    password    TEXT NOT NULL,
    steps       INTEGER NOT NULL DEFAULT 0,
    weeklysteps INTEGER NOT NULL DEFAULT 0,
    changelogin TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_weeklysteps ON users (weeklysteps DESC);

CREATE TABLE IF NOT EXISTS tokens (
    login TEXT PRIMARY KEY,
    token TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS currentleader (
    currentleader TEXT NOT NULL
);
