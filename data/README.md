# Planting Life Data
Organizes information about native plants to answer questions like:
- What plants are native near me?
- Will it grow in the shade?
- How tall is it?
- Are deer going to eat it?
- Will it take over my garden?

## High-Level Design
Data is collected using an [ETL](https://en.wikipedia.org/wiki/Extract%2C_transform%2C_load) process driven by [Luigi](https://github.com/spotify/luigi). Information from "expert sources" like [USDA](https://plants.usda.gov) and [Wildflower.org](https://wildflower.org) is preferred over LLM sources. LLMs are used to answer questions that don't have good expert sources.

Images come from [iNaturalist](https://inaturalist.org) and [Flickr](https://flickr.com). An automatic attempt is made to choose a reasonable images based on votes/views, but the image picker UI can be used to choose better options. Better options have been chosen for most plants native to the Mid West, but not much has been done for other areas of the country.

`generate_all_sql.py` is an orchestration script which generates a SQL diff to update the local database to the new, updated data.

## Setup
- Install [uv](https://docs.astral.sh/uv/).
- Copy .env.example to .env, fill in your OpenAI and Flickr API keys
- If generating SQL, start the local database (see backend docs)
## Usage
```bash
# Collect data about the plants in short.txt
uv run generate_plants_csv.py data/in/scientific-names/short.txt

# Print the collected data to the terminal
cat data/out/plants-short.csv

# Generate a diff to update local database
# If encountering "Too many files open", run: ulimit -n 2048
uv run generate_all_sql.py data/in/all.txt

# Run tests
uv run pytest

# Run image picker
uv run image_picker.py data/in/scientific-names/all.txt

# Run one task directly, usually for debugging
PYTHONPATH=. uv run luigi --local-scheduler --module tasks.datasources.usda.location TransformPlantZipcodes --scientific-name "ilex verticillata"
```

