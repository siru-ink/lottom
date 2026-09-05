-- Create relations needed for shopping lists management
CREATE TABLE IF NOT EXISTS lists (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS items (
    id SERIAL PRIMARY KEY,
    list_id INTEGER REFERENCES lists(id) NOT NULL,
    en_name TEXT NOT NULL,
    zh_name TEXT NOT NULL,
    de_name TEXT NOT NULL,
    img_path TEXT UNIQUE,
    estimated_euro_price INTEGER NOT NULL
);

-- Create relations needed for authentication and session management
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    default_list_id INTEGER REFERENCES lists(id) NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    start TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Create relations needed for shared lists access
CREATE TABLE IF NOT EXISTS roles (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS lists_users_mapping (
    list_id INTEGER REFERENCES lists(id) NOT NULL,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    role_id INTEGER REFERENCES roles(id) NOT NULL
);

-- Create relations needed for storing known shopping list items
CREATE TABLE IF NOT EXISTS prefill_items (
    id SERIAL PRIMARY KEY,
    en_name TEXT UNIQUE NOT NULL,
    zh_name TEXT UNIQUE NOT NULL,
    de_name TEXT UNIQUE NOT NULL,
    euro_price INTEGER NOT NULL
);

-- Create default user list access roles
INSERT INTO roles (id, name) VALUES (1, 'owner'), (2, 'editor'), (3, 'viewer');
