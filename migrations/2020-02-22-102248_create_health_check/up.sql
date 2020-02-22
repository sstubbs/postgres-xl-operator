CREATE TABLE ping
(
    id        SERIAL PRIMARY KEY,
    t_ins     timestamptz NOT NULL DEFAULT now()
)