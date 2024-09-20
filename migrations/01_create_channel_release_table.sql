CREATE TABLE releases (
    id INTEGER PRIMARY KEY NOT NULL,
    version TEXT NOT NULL,
    hash TEXT NOT NULL,
    channel_id INTEGER NOT NULL,
    created_at INTEGER  DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at INTEGER  DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES channel (id)
);

CREATE TRIGGER releases
AFTER UPDATE OF id, version, hash, channel_id
ON releases
FOR EACH ROW BEGIN UPDATE releases
  SET updated_on = DATETIME('NOW')
  WHERE rowid = new.rowid;
END
