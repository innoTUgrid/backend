from typing import Self
import requests
import polars as pl
from scraper import Scraper
from datetime import datetime, timezone

class SmardScraper(Scraper):
    TIMESTAMPS_URL = "https://www.smard.de/app/chart_data/%s/%s/index_%s.json"
    COLUMNS = ["timestamp", "price"]

    def __init__(self) -> None:
        super().__init__()
        self.dataframe = None
        """Raw dataframe for a given year"""
        self.last_scrape = None
        """Timestamp of the last scrape"""

    def scrape(self, from_timestamp: int, to_timestamp: int, filter_value="4169", region="DE", resolution="hour"):
        timestamps_url = self.TIMESTAMPS_URL % (filter_value, region, resolution)
        timestamp_response = requests.get(timestamps_url)
        timestamps = timestamp_response.json().get("timestamps", [])
        filtered_timestamps = [timestamp for timestamp in timestamps if timestamp >= from_timestamp and timestamp <= to_timestamp] 
        timeseries_responses = []
        for timestamp in filtered_timestamps:
            timeseries_url = f"https://www.smard.de/app/chart_data/{filter_value}/{region}/{filter_value}_{region}_{resolution}_{timestamp}.json"
            print(timeseries_url)
            timeseries_response = requests.get(timeseries_url)
            timeseries_responses.append(timeseries_response.json()["series"])
            print(timeseries_response.json()["series"])
            print(len(timeseries_response.json()["series"]))
        self.last_scrape = datetime.now(timezone.utc)
        self.dataframe = pl.DataFrame(timeseries_responses)
        return self 
    
        
    def filter(self, **kwargs):
        """Filter smard market data by range"""
        pass 


if __name__ == "__main__":
    test_start = int(datetime.fromisoformat("2020-01-01").timestamp())
    test_end = int(datetime.fromisoformat("2021-01-01").timestamp())
    print(test_start, test_end)
    scraper = SmardScraper().scrape(
        test_start,
        test_end
    )
    print(scraper.dataframe)


    
