# Backend services for innoTUgrid

The backend consists of multiple independent services each providing different functionality.

### how to run this code
1. start the container running the database detached from the terminal
`docker compose up db -d`
2. check if container is running
`docker ps`
3. execute migrations
`sqlx migrate run`
4. compile & start backend server
`cargo run`
5. test API endpoint
`curl -X GET localhost:3000`

### access database
- access the database container
`docker exec -it timescaledb bash`
- log into database
`psql -U ${POSTGRES_DB_USER} -d ${POSTGRES_DB_NAME}`

### check API documentation
Open the `documentation/inno2grid_api_documentation.yaml` using the [Online Swagger Editor](https://editor.swagger.io/).
If you want to run API calls from Swagger you might need to run it locally. Follow the [Swagger Docs to set up a localhost using Docker](https://swagger.io/docs/open-source-tools/swagger-ui/usage/installation/).