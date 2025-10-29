CREATE TABLE with_json_non_nullable (
    id   INT PRIMARY KEY,
    data JSONB NOT NULL
);

CREATE TABLE with_json_nullable (
    id   INT PRIMARY KEY,
    data JSONB
);
