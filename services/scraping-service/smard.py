import requests
from scraper import Scraper
from datetime import datetime, timezone


class SmardScraper(Scraper):
    """Fetch electricity market price data from the SMARD API"""
    BASE_URL = "https://www.smard.de/app/chart_data/"
    TIMESTAMPS_URL = BASE_URL + "{filter_value}/{region}/index_{resolution}.json"
    TIMESERIES_URL = BASE_URL + "{filter_value}/{region}/{filter_value}_{region}_{resolution}_{timestamp}.json"
    DATA_SERVICE_TS_ENDPOINT = "http://localhost:3000/v1/ts/"
    DATA_SERVICE_META_ENDPOINT = "http://localhost:3000/v1/meta/"
    SMARD_UNIT = "eur/mwh"
    SMARD_IDENTIFIER = "smard_market_price"

    def __init__(self) -> None:
        super().__init__()
        self.responses = None
        """Raw responses for a given year"""
        self.datapoints = []
        """Formatted datapoints for a given year"""
        self.last_scrape = None
        """Timestamp of the last scrape"""

    def scrape(self, from_timestamp: int, to_timestamp: int, filter_value="4169", region="DE", resolution="hour"):
        """Scrape the SMARD API for the given parameters and store a list of datapoints"""
        timestamps_url = self.TIMESTAMPS_URL.format(
            filter_value=filter_value,
            region=region,
            resolution=resolution
        )
        timestamp_response = requests.get(timestamps_url)
        timestamps = timestamp_response.json().get("timestamps", [])
        filtered_timestamps = [timestamp for timestamp in timestamps if
                               from_timestamp <= timestamp <= to_timestamp]
        timeseries_responses = []
        for timestamp in filtered_timestamps:
            url = self.TIMESERIES_URL.format(
                filter_value=filter_value,
                region=region,
                resolution=resolution,
                timestamp=timestamp
            )
            timeseries_response = requests.get(url)
            timeseries_responses.append(timeseries_response.json()["series"])
        self.last_scrape = datetime.now(timezone.utc)
        self.responses = timeseries_responses[0]

        for response in self.responses:
            ts, value = response
            ts = ts / 1000
            ts = datetime.fromtimestamp(ts, tz=timezone.utc).isoformat()
            self.datapoints.append((ts, value))
        return self

    def check_if_series_exists(self):
        pass

    def write(self):
        pass


if __name__ == "__main__":
    test_start = int(datetime.fromisoformat("2020-01-01").timestamp()) * 1000
    test_end = int(datetime.fromisoformat("2020-12-31").timestamp()) * 1000
    scraper = SmardScraper().scrape(
        test_start,
        test_end
    )
