# Actix-web + Postgres 
### using shared state
---
## Watch the video 
  https://youtu.be/TVT7VHynTCc

  `CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL
);
`

Example row in database :<br>
`{"id":99,"name":"Jacddd55","email":"jack9@example.com"}`<br>

add this to your ~/.zshrc or ~/.bashrc<br>
  `export DATABASE_URL="postgres://new_user:new_password@localhost:5432/new_db"`
Use your own credentials though!
