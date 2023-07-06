from bs4 import BeautifulSoup
import re
from address_parser import Parser
import requests
from requests.structures import CaseInsensitiveDict
import urllib.parse
import os

us_state_to_abbrev = {
    "Alabama": "AL",
    "Alaska": "AK",
    "Arizona": "AZ",
    "Arkansas": "AR",
    "California": "CA",
    "Colorado": "CO",
    "Connecticut": "CT",
    "Delaware": "DE",
    "Florida": "FL",
    "Georgia": "GA",
    "Hawaii": "HI",
    "Idaho": "ID",
    "Illinois": "IL",
    "Indiana": "IN",
    "Iowa": "IA",
    "Kansas": "KS",
    "Kentucky": "KY",
    "Louisiana": "LA",
    "Maine": "ME",
    "Maryland": "MD",
    "Massachusetts": "MA",
    "Michigan": "MI",
    "Minnesota": "MN",
    "Mississippi": "MS",
    "Missouri": "MO",
    "Montana": "MT",
    "Nebraska": "NE",
    "Nevada": "NV",
    "New Hampshire": "NH",
    "New Jersey": "NJ",
    "New Mexico": "NM",
    "New York": "NY",
    "North Carolina": "NC",
    "North Dakota": "ND",
    "Ohio": "OH",
    "Oklahoma": "OK",
    "Oregon": "OR",
    "Pennsylvania": "PA",
    "Rhode Island": "RI",
    "South Carolina": "SC",
    "South Dakota": "SD",
    "Tennessee": "TN",
    "Texas": "TX",
    "Utah": "UT",
    "Vermont": "VT",
    "Virginia": "VA",
    "Washington": "WA",
    "West Virginia": "WV",
    "Wisconsin": "WI",
    "Wyoming": "WY",
    "District of Columbia": "DC",
    "American Samoa": "AS",
    "Guam": "GU",
    "Northern Mariana Islands": "MP",
    "Puerto Rico": "PR",
    "United States Minor Outlying Islands": "UM",
    "U.S. Virgin Islands": "VI",
}

parser = Parser()
html_doc = open("nurseries.xml")
out_file = open("nurseries.csv", "w")
geoapify_api_key = os.environ['GEOAPIFY_API_KEY']

# Assuming 'html_doc' is your HTML document
soup = BeautifulSoup(html_doc, 'html.parser')

# Find all 'p' elements with class 'foo'
p_elements = soup.find_all('p', class_='has-text-align-center')

out_file.write('"name", "url", "address", "city", "state", "zip", "latitude", "longitude"\n')

print("\n\n\n\n")
# Iterate through each element
for element in p_elements:

    strong_element = element.find('strong')

    # Find the 'a' element within the 'p' element
    a_element = strong_element.find('a')

    # Extract the URL and name
    if a_element:
        name = a_element.get_text()
        url = a_element.get('href').replace(" ", "").replace("%20", "")
    else:
        name = strong_element.string
        url = ''

    # squish whitespace
    name = re.sub(r'\s+', ' ', name)
    print(f'Name: {name}')
    print(f'URL: {url}')

    # Extract the street address, city, state, zip, and phone
    # These are located after the 'strong' and 'br' elements
    # Split the remaining text by the 'br' elements and strip any leading/trailing whitespace
    address_parts = [''] * 4
    i = 0
    for sibling in element.find('strong').next_siblings:
        if sibling.name == 'br':
            i += 1
        else:
            # squish whitespace, ignore everything after <
            address_parts[i] += re.sub(r'\s+', ' ', str(sibling)).split("<")[0].strip()

    if address_parts[0].startswith("("):
        print(f'Skipping: {name}, missing address')
        continue
    #print(f'address_parts: {address_parts}')
    street_address = address_parts[0]
    print(f'Street Address: {street_address}')

    city_state_zip = address_parts[1]

    # Further split the city, state, and zip
    #print(f'city_state_zip {city_state_zip}')
    #parsed = parser.parse(city_state_zip)
    city_state_zip_parts = city_state_zip.split(',')
    if len(city_state_zip_parts) < 2:
        print(f'Skipping: {name}, missing city')
        continue
    city = city_state_zip_parts[0]
    print(f'City: {city}')

    state_zip = city_state_zip_parts[1].strip()
    state_zip_parts = state_zip.rsplit(" ", 1)
    #print(f'state_zip: {state_zip}')
    state = us_state_to_abbrev.get(state_zip_parts[0], "")
    print(f'State: {state}')
    zip_code = state_zip_parts[1].strip() if len(state_zip_parts) > 1 else ''
    # ignore - and anything after
    zip_code = zip_code.split("-")[0]
    print(f'ZIP: {zip_code}')

    print()

    # These were missing zipcodes, so I just looked them up and hardcoded them.
    # Making it automated in case I regenerate this in the future
    if street_address == "2230 Valley Hwy" and city == "Charlotte" and state == "MI":
        zip_code = "48813"
    if street_address == "1480 County Rd 90" and city == "Independence" and state == "MN":
        zip_code = "55359"
    if street_address == "491 State Highway 46" and city == "Amery" and state == "WI":
        zip_code = "54001"

    if street_address == "" or city == "" or state == "":
        continue # these fields are required

    if "PO Box" in street_address or "P.O. Box" in street_address or "By Appointment Only" in street_address:
        continue # I want an address someone can drive to

    # No url is okay, but don't return urls that don't work
    valid_url = ""
    try:
        response = requests.get(url, timeout=10)
        if response.status_code == 200:
            valid_url = url
    except:
        # do nothing on failure, valid_url just isn't set
        print(f"Invalid url: {url}")

    # Lookup latitude & longitude
    query = f"{street_address}, {city}, {state} {zip_code}, United States"
    query = urllib.parse.quote(query)
    geoapify_url = f"https://api.geoapify.com/v1/geocode/search?text={query}&apiKey={geoapify_api_key}&format=json"
    print(geoapify_url)

    headers = CaseInsensitiveDict()
    headers["Accept"] = "application/json"

    resp = requests.get(geoapify_url, headers=headers)
    if resp.status_code != 200:
        print(f"geoapify error: {resp.status_code}")
    else:
        print(resp.content)
        data = resp.json()
        # Extract the first result from the data
        if len(data["results"]) > 0:
            result = data["results"][0]
            
            # Extract the latitude and longitude of the result
            latitude = result["lat"]
            longitude = result["lon"]
        else:
            latitude = ""
            longitude = ""

    print(f"{latitude}, {longitude}")

    out_file.write(f'"{name}", "{valid_url}", "{street_address}", "{city}", "{state}", "{zip_code}", "{latitude}", "{longitude}"\n')

