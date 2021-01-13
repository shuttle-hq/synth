CREATE TABLE generations (
       namespace VARCHAR NOT NULL,
       generation INTEGER NOT NULL,
       timestamp TIMESTAMP NOT NULL,
       PRIMARY KEY (namespace, generation)
);
