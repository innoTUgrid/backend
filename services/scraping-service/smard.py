import logging

import requests
import schedule
import time

from scraper import Scraper
from datetime import datetime, timezone, timedelta
from dateutil.parser import parse

from logging import getLogger
import os

logger = getLogger(__name__)
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s.%(msecs)03d %(levelname)s [%(filename)s:%(lineno)d] %(message)s',
    datefmt='%Y-%m-%dT%H:%M:%S',
)


class SmardScraper(Scraper):
    """Fetch electricity market price data from the SMARD API"""
    BASE_URL = "https://www.smard.de/app/chart_data/"
    TIMESTAMPS_URL = BASE_URL + "{filter_value}/{region}/index_{resolution}.json"
    TIMESERIES_URL = BASE_URL + "{filter_value}/{region}/{filter_value}_{region}_{resolution}_{timestamp}.json"
    SMARD_UNIT = "eur/mwh"
    SMARD_IDENTIFIER = "smard_market_price"

    def __init__(self) -> None:
        super().__init__()
        self.responses = None
        """Raw responses for a given year"""
        self.datapoints = []
        """Formatted datapoints for a given year"""
        self.data_service_max_timestamp = None
        """Current oldest timestamp of the series in the data service"""
        self.smard_min_timestamp = datetime.fromisoformat("2019-01-01T00:00:00Z").timestamp() * 1000
        self.smard_max_timestamp = datetime.fromisoformat("2021-01-01T00:00:00Z").timestamp() * 1000
        self.API_URL = os.getenv('API_URL', 'http://localhost:3000')
        self.DATA_SERVICE_TS_ENDPOINT = f"{self.API_URL}/v1/ts/"
        self.DATA_SERVICE_META_ENDPOINT = f"{self.API_URL}/v1/meta/"

    def run(self, filter_value="4169", region="DE", resolution="hour"):
        """Scrape the SMARD API for the given parameters and store a list of datapoints"""

        if self.series_exists():
            metadata = self.get_series_metadata()
            if metadata["max_timestamp"]:
                # timestamp might contain Z so use dateutil
                # convert the data service timestamp to the format used by SMARD for convenience
                self.data_service_max_timestamp = parse(metadata["max_timestamp"]).timestamp() * 1000
                logger.info("Max timestamp in data service is %s" % metadata["max_timestamp"])
            else:
                self.data_service_max_timestamp = datetime.fromisoformat("1970-01-01T00:00:00Z").timestamp() * 1000
                logger.info("Can't find max timestamp in data service. Set to default")
        else:
            # use the unix epoch as a default value
            logger.info("%s does not exist in data service. Creating" % self.SMARD_IDENTIFIER)
            self.data_service_max_timestamp = datetime.fromisoformat("1970-01-01T00:00:00Z").timestamp() * 1000
            # create the series
            response = requests.post(
                self.DATA_SERVICE_META_ENDPOINT,
                json={
                    "identifier": self.SMARD_IDENTIFIER,
                    "unit": self.SMARD_UNIT
                },
            )
            if response.status_code == 200:
                logger.info("Successfully created series in data service")
            else:
                logger.error("Failed to create series in data service")

        timestamps_url = self.TIMESTAMPS_URL.format(
            filter_value=filter_value,
            region=region,
            resolution=resolution
        )
        timestamp_response = requests.get(timestamps_url)
        timestamps = timestamp_response.json().get("timestamps", [])
        logger.info("Fetched %d timestamps from SMARD" % len(timestamps))
        timeseries_responses = []
        filtered_timestamps = [timestamp for timestamp in timestamps if
                               self.smard_min_timestamp < timestamp < self.smard_max_timestamp]
        logger.info("Filtered %d timestamps that are already in the data service or excluded by configuration" % len(
            filtered_timestamps))
        for timestamp in filtered_timestamps:
            url = self.TIMESERIES_URL.format(
                filter_value=filter_value,
                region=region,
                resolution=resolution,
                timestamp=timestamp
            )
            timeseries_response = requests.get(url)
            logger.info("%d for timestamp %s from SMARD" % (
            timeseries_response.status_code, datetime.fromtimestamp(timestamp / 1000, tz=timezone.utc).isoformat()))
            timeseries_responses.append(timeseries_response.json()["series"])
        logger.info("Trying to write %d responses" % len(timeseries_responses))
        for responses in timeseries_responses:
            for response in responses:
                ts, value = response
                ts = ts / 1000
                ts = datetime.fromtimestamp(ts, tz=timezone.utc).isoformat()
                self.datapoints.append((ts, value))

        data_json = [
            {
                "identifier": self.SMARD_IDENTIFIER,
                "timestamp": datapoint[0],
                "value": datapoint[1]
            }
            for datapoint in self.datapoints
        ]

        response = requests.post(
            self.DATA_SERVICE_TS_ENDPOINT,
            json={"timeseries": data_json}
        )
        if response.status_code == 200:
            logger.info("Successfully wrote %d datapoints to data service" % len(self.datapoints))
        else:
            logger.error("Failed to write data to data service. Data service responded with %d %s" % (
                response.status_code, response.text))

    def series_exists(self):
        """Fetch the current metadata from the data service"""
        response = requests.get(
            self.DATA_SERVICE_META_ENDPOINT + self.SMARD_IDENTIFIER + "/"
        )
        return response.status_code == 200

    def get_series_metadata(self):
        """Fetch the current state of the series from the dataservice"""
        response = requests.get(
            self.DATA_SERVICE_META_ENDPOINT + self.SMARD_IDENTIFIER + "/"
        )
        return response.json()


if __name__ == "__main__":
    scraper = SmardScraper()

    # run scraper once, then every hour
    scraper.run()
    schedule.every().hour.do(scraper.run)

    while True:
        schedule.run_pending()
        time.sleep(1)
