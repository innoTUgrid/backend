# Architecture and Backend notes


## General notes

* we decided on a microservice based architecture because of initially unclear scalability and performance requirements
* it generally is easier to start with a distributed architecture than to split an already developed monolith into individual services
* this architecture is very extensible and allows for straightforward horizontal scaling by adding more instances of each service
* non-essential features (e.g. prediction, simulation) can be added as individual services after the fact building on the services we already implemented

## Data service

* data service is the main backend service we implemented
* it provides a http api for reading and writing various timeseries and KPI
* the implementation language is rust
* rust was chosen because of correctness, performance and strong typing guarantees
* rust is very energy efficient and has a low CPU and memory footprint
* this makes it very cheap to host on cloud providers 
* the service itself is built on top of axum, which is a micro-framework similar to flask
* most of the logic of interacting with the database is implemented via sql
* we can insert roughly 150k rows per second

## SMARD service 

* this service is implemented in python and responsible for talking to the SMARD API for electricity market prices
* it fetches data as defined by configuration from SMARD and writes them to the data service to provide data for e.g. cost savings KPI 
* we chose python because of development velocity and because performance was not critical, as the scraper only runs once per hour and and is push-based 


## Database

* standard postgres deployment with timescaledb plugin
* the timescaledb plugin allows for effective and fast workflows for timeseries, e.g. aggregation and timestamp bucketing
* we went with an extremely flexible schema which is extensible and doesn't make a lot of assumption on how the data is going to look like


## Deployment

* each service (including the frontend) is provided as a Docker container
* for orchestration, we currently use docker-compose
* this is fine for single node or local deployments, but would require a switch to either docker swarm or kubernetes for a distributed deployment
* having each service as an independent container makes the deployment straightforward with any common cloud provider
* we chose this setup because it is vendor-agnostic and flexible

## DevOps

* we set up comprehensive CI/CD for the data service
    * enforce code format and style with a linter
    * run tests on every commit
    * build executable artifact on every commit
    * deployment and development environment are exactly the same

# Challenges (Backend)

* technology choices with respect to database, containerization, implementation languages
* schema in data service had to be refactored multiple times until we covered everything
    * rust as a language helped immensely because of strong typing guarantees and compiled time checked queries
    * while it was annoying to refactor the schema multiple times, the actual refactoring went smoothly
* lack of domain expertise  
    * took multiple tries to get e.g. energy production calculations from power measurements right
    * hard to validate KPI calculations because of lack of intuition for scale of results (e.g. is this co2 emission number realistic?) which made debugging hard
    * this was exacerbated by relatively poor data documentation 
* typical frontend-backend communication issues
    * e.g. frontend means and wants A but backend understands B and implements B which eventually gets refactored into A


# Future Work (Backend)

## Observability

* each service currently has a simple logging setup
* these logs could be aggregated, enriched with relevant metrics (throughput, number of queries, number of requests etc.) and processed by another service
* this would allow one single backend-centric viewpoint into the entire system for observability 
* because of relatively low number of services distributed tracing would probably be overkill

## Prediction and Simulation

* was only briefly discussed with chantal
* however, based on data from the data service could make sense for forward-looking analysis (e.g. expected cost savings, expected) or simulation (what-if scenarios)
* could implement various predictive models and simulations independently as individual services
* these would consume data from the data service and could then provide model data back to the frontend
* most suitable implementation language would probably be python because of strong stats/ml ecosystem
* microservices are very neat here because they allow for isolation with respect to different modeling / simulation approaches
    * this makes separation from the actual data very clean and would allow for independent development
    * e.g. a single view in the frontend per hypothetical scenario



