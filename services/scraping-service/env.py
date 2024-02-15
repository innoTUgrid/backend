from dotenv import load_dotenv
import os

load_dotenv()

API_URL = os.getenv("API_URL", "http://host.docker.internal:3000")
