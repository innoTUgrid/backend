# Scraping Service

The scraping service provides functionality to scrape various external data sources, 
such as the PyPSA project. 
Each data source is implemented as an individual scraper.

# Setup and Dependency Management 

We use poetry for Python dependency management.

Install poetry
```bash
pip install poetry
```
Install project dependencies 

```bash
poetry install
```

Activate the virtual environment

```bash
source $(poetry env info -p)/bin/activate
```





