-- src/sql2/create_project.sql
-- Create new project namespace

INSERT INTO project_ns (pns_name) 
VALUES (?1);
