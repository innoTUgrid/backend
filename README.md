# Backend services for innoTUgrid

The backend consists of multiple independent services each providing different functionality.

### how to run this code
1. start the container running the database detached from the terminal
`docker compose up db`
2. check if container is running
`docker ps`
3. execute migrations
`sqlx migrate run`
4. compile & start backend server
`cargo run`
5. test API endpoint using *curl*
`curl -X GET localhost:3000`

### access database
- access the database container
`docker exec -it timescaledb bash`
- log into database
`psql -U ${POSTGRES_DB_USER} -d ${POSTGRES_DB_NAME}`