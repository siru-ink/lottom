-- Add default admin user
INSERT INTO lists (id, name) VALUES (1, 'admin_default');
INSERT INTO users (name, password, default_list_id) VALUES ('admin', 'test', 1);
