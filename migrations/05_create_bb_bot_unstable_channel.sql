INSERT INTO release_channel (name, project_id) VALUES (
    "unstable",
    (SELECT id from projects where name = "BB bot")
);
