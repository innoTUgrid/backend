"""Scraping module for pypsa technology data"""
import requests
import polars as pl
from scraper import Scraper
from io import StringIO 
from datetime import datetime, timezone

class PypsaScraper(Scraper):
    """Scrape various technology data from the PyPSA project"""
    BASE_URL = "https://raw.githubusercontent.com/PyPSA/technology-data/master/outputs/costs_%d.csv"
    COLUMNS = ["technology", "parameter", "value", "unit"]
    PYPSA_YEARS = [
        2020, 
        2025, 
        2030, 
        2035, 
        2040, 
        2045, 
        2050
    ]
    def __init__(self) -> None:
        super().__init__()
        self.dataframe = None
        """Raw dataframe for a given year"""
        self.year = None
        """Year of the current data"""
        self.last_scrape = None
        """Timestamp of the last scrape"""

    def __generate_url(self, year):
        if year not in self.PYPSA_YEARS:
            raise ValueError(f"PyPSA only supports data for years {self.PYPSA_YEARS}")
        url = self.BASE_URL % year
        return url
    
    def get_year(self, year):
        """Fetch a single year of PyPSA technology data"""
        url = self.__generate_url(year=year)
        self.year = year
        self.last_scrape = datetime.now(timezone.utc)
        response = requests.get(url=url)
        if response.status_code == 200:
            response_data = StringIO(response.text)
            df = pl.read_csv(response_data)
            self.dataframe = df.with_columns(pl.lit(self.year).alias("year"))
        return self
        
    def filter(self, **kwargs):
        """Filter technology data by keywords"""
        self.acc = self.dataframe
        for column, value in kwargs.items():
            self.acc = (
                self.acc
                .select(self.COLUMNS)
                .filter(pl.col(column) == value)
            )
        return self.acc 
    

if __name__ == "__main__":
    scraper = PypsaScraper()
    print(scraper.get_year(year=2025).filter(parameter="efficiency", technology="coal"))
    