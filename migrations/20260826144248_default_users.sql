-- Add default admin user
INSERT INTO lists (id, name) VALUES (1, 'admin_default');
INSERT INTO users (id, name, password, default_list_id) VALUES (1, 'admin', 'test', 1);
