INSERT INTO release_channel (name, project_id) VALUES (
    "stable",
    (SELECT id from projects where name = "BB bot")
);
