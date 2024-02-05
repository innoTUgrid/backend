# Backend services for innoTUgrid

The backend consists of multiple independent services each providing different functionality.

### how to run this code (docker)

Remove any existing containers and volumes:
```bash
docker compose down -v
```

Pull our prebuilt image from ghcr.io: 
```bash
docker compose pull
```

Run the containers:
```bash
docker compose up -d
```

### how to run this code locally

1. start the container running the database detached from the terminal
```bash 
docker compose up -d db
```

2. if running it for the **first time** or after recreating the database ... 
```bash 
./scripts/init-db.sh
```

3. then:
`docker compose up -d`


#### **drop database** e.g. in case initial data has been updated and needs to reinitialize (see step 2. above)
1. remove the database container
```bash
docker compose down db -v
```
2. then start the database container detached from the terminal
```bash
docker compose up db -d
```

2. then run the initialization script
```bash 
./scripts/init-db.sh
```


#### **finally** check ping endpoint in terminal ...
```bash
curl -X GET localhost:3000
```
#### ... or call it via a browser and remember to stay caffeinated ;-)
```bash
localhost:3000
```

### optionally **access the database** to run SQL queries directly
1. access the database container
```bash
docker exec -it timescaledb bash
```
2. log into database (replace environemnt variables with parameters found in `.env` file)
```bash
psql -U ${POSTGRES_DB_USER} -d ${POSTGRES_DB_NAME}
```

### check API documentation
Open the `documentation/inno2grid_api_documentation.yaml` using the [Online Swagger Editor](https://editor.swagger.io/).
If you want to run API calls from Swagger you might need to run it locally. Follow the [Swagger Docs to set up a localhost using Docker](https://swagger.io/docs/open-source-tools/swagger-ui/usage/installation/).