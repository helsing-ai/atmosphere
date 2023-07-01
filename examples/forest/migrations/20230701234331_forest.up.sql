CREATE TABLE forest (
    id       INT PRIMARY KEY,
    name     TEXT NOT NULL,
    location TEXT NOT NULL
);

CREATE TABLE tree (
    id        INT PRIMARY KEY,
    forest_id INT NOT NULL REFERENCES forest(id) ON DELETE CASCADE
);
