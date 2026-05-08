CREATE TABLE IF NOT EXISTS sensors (
    id SERIAL PRIMARY KEY,
    ip VARCHAR(45)
);

CREATE TABLE IF NOT EXISTS clients (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50),
    "timestamp" TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id CHAR(32) PRIMARY KEY,
    starttime TIMESTAMP WITH TIME ZONE,
    endtime TIMESTAMP WITH TIME ZONE,
    sensor INTEGER REFERENCES sensors(id),
    ip VARCHAR(45),
    termsize VARCHAR(7) DEFAULT '',
    client INTEGER REFERENCES clients(id)
);

CREATE TABLE IF NOT EXISTS auth (
    id SERIAL PRIMARY KEY,
    session CHAR(32) REFERENCES sessions(id),
    success BOOLEAN DEFAULT FALSE,
    username TEXT,
    password TEXT,
    "timestamp" TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS input (
    id SERIAL PRIMARY KEY,
    session CHAR(32) REFERENCES sessions(id),
    "timestamp" TIMESTAMP WITH TIME ZONE,
    realm VARCHAR(20),
    input TEXT,
    success BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS ttylog (
    id SERIAL PRIMARY KEY,
    session CHAR(32) REFERENCES sessions(id),
    ttylog VARCHAR(100)
);

CREATE TABLE IF NOT EXISTS downloads (
    id SERIAL PRIMARY KEY,
    session CHAR(32) REFERENCES sessions(id),
    "timestamp" TIMESTAMP WITH TIME ZONE,
    url TEXT,
    outfile TEXT,
    shasum VARCHAR(64)
);

CREATE TABLE IF NOT EXISTS keyfingerprints (
    id SERIAL PRIMARY KEY,
    session CHAR(32) REFERENCES sessions(id),
    username TEXT,
    fingerprint VARCHAR(100)
);
