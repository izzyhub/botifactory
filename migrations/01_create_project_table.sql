CREATE TABLE projects (
    id INTEGER PRIMARY KEY NOT NULL,
    created_at INTEGER  DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at INTEGER  DEFAULT CURRENT_TIMESTAMP NOT NULL,
    name TEXT NOT NULL
);


CREATE TRIGGER projects_updated_at
AFTER UPDATE OF id, version, hash, channel_id
ON projects
FOR EACH ROW BEGIN UPDATE projects
  SET updated_at = DATETIME('NOW')
  WHERE rowid = new.rowid;
END
