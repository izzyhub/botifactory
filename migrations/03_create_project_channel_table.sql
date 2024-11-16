CREATE TABLE release_channel (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    project_id INTEGER NOT NULL,
    created_at INTEGER DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at INTEGER DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects (id),
    UNIQUE (name, project_id)
);

CREATE TRIGGER release_channel_updated_at
AFTER UPDATE OF id, version, hash, channel_id
ON release_channel
FOR EACH ROW BEGIN UPDATE release_channel
  SET updated_at = DATETIME('NOW')
  WHERE rowid = new.rowid;
END
