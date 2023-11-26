# Backend services for innoTUgrid

The backend consists of multiple independent services each providing different functionality.

### how to run this code

- start the container running the database detached from the terminal
`docker compose up db`
- check if container is running
`docker ps`
- execute migrations
`sqlx migrate run`
- compile & start backend server
`cargo run`
- test API endpoint using ~curl~
`curl -X GET localhost:3000`

- access the database container
`docker exec -it timescaledb bash`
- log into database
`psql -U ${POSTGRES_DB_USER} -d ${POSTGRES_DB_NAME}`