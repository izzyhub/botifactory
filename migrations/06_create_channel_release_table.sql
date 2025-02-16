CREATE TABLE releases (
    id INTEGER PRIMARY KEY NOT NULL,
    version TEXT NOT NULL,
    hash TEXT NOT NULL,
    path TEXT NOT NULL,
    channel_id INTEGER NOT NULL,
    created_at INTEGER  DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at INTEGER  DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES release_channel (id),
    UNIQUE (hash, version, channel_id)

);

CREATE TRIGGER releases_updated_at
AFTER UPDATE OF id, version, hash, channel_id
ON releases
FOR EACH ROW BEGIN
  UPDATE releases
  SET updated_at = DATETIME('NOW')
  WHERE rowid = new.rowid;
END
