# Backend services for innoTUgrid

The backend consists of multiple independent services each providing different functionality.

### how to run this code

1. start the container running the database detached from the terminal
`docker compose up db -d`

2. if running it for the **first time** or after recreating the database ... 
`sqlx database create`

3. ... otherwise simply do:
`docker compose up`

#### **drop database** e.g. in case initial data has been updated and needs to reinitialize (see step 2. above)
1. check if container hosting the database is running
`docker ps`
2. alternatively start it with
`docker compose up db -d`
3. then drop & recreate empty database
`sqlx database drop`

#### **finally** check ping endpoint in terminal ...
`curl -X GET localhost:3000`
#### ... or call it via a browser and remember to stay caffeinated ;-)
`localhost:3000`

### optionally **access the database** to run SQL queries directly
1. access the database container
`docker exec -it timescaledb bash`
2. log into database (replace environemnt variables with parameters found in `.env` file)
`psql -U ${POSTGRES_DB_USER} -d ${POSTGRES_DB_NAME}`

### check API documentation
Open the `documentation/inno2grid_api_documentation.yaml` using the [Online Swagger Editor](https://editor.swagger.io/).
If you want to run API calls from Swagger you might need to run it locally. Follow the [Swagger Docs to set up a localhost using Docker](https://swagger.io/docs/open-source-tools/swagger-ui/usage/installation/).