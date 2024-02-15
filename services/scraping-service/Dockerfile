FROM python:3.11-slim-bullseye

# Set the environment variables
# The API_URL is the URL of the backend service (without trailing slash)
ENV API_URL="http://localhost:3000"

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --assume-yes --no-install-recommends \
      gcc python3-dev
WORKDIR /app
ADD . /app
RUN pip install poetry

RUN poetry config virtualenvs.create false && poetry install --no-interaction --no-ansi
CMD poetry run python smard.py
