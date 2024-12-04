CREATE TABLE
  spell (
    id bigserial primary key,
    name VARCHAR,
    damage INT not null,
    created_at TIMESTAMPTZ NOT NULL default now (),
    updated_at TIMESTAMPTZ NOT NULL default now ()
  );

-- BTree and LSM Tree
-- GIN and GIST
/*
Primary Key already has a BTree index -> spell_pkey

explain analyze select damage from spell where id = 2;
-> Index Scan using spell_pkey on spell
 */
--- 
/*
Damage is not indexed

explain analyze select damage from spell where name = 'Fireball';
-> Seq Scan on spell
 */

CREATE INDEX spell_name on spell (name);