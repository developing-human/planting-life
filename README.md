# Planting Life
Planting Life is a web application designed to make it simple to discover native plants that will thrive based on location, sunlight, and moisture.  The project's mission is to make this simple in order to encourage decisions which help the ecosystem.  It's hosted at https://planting.life.

## Why?
Native plants can be eaten by local insects, which can be eaten by local birds, which leads to a thriving ecosystem.  Non-native plants often (though not always) lack these benefits.

## How can you help?
TODO: Describe how people can help in both technical and non-technical ways.

## How does it work?
The suggested plants come from a mixture of ChatGPT, Flickr images, and USDA data.  Over time, the project is likely to use additional structured data.

#### Selecting plants
A ChatGPT prompt is built which includes the location and conditions, which responds with a list of plant names.  This prompt has proven very reliable for suggesting plants which are native, but often suggests plants which would do better in other conditions (ex: suggests a plant for full shade which prefers full sun).

Next, ChatGPT is queried again for each plant asking for the conditions it will do best in.  As a separate query, conditions can be determined by ChatGPT more reliably.  If a plant passes this filter, it will be sent to the front end, but first...

#### Hydrating plants
After being selected, the plant's data model is sparsely populated and it must be hydrated.  This includes:
1. Searching flickr for an image
2. Looking up related links for Wikipedia and USDA
3. Looking up characteristics like when it blooms, height, and width/spread
4. Rating the plant on various metrics (ecosystem support, deer resistance, aggressive spread)
5. Generating "highlights" out of the ratings

#### Caching
Nearly everything mentioned above is cached in a relational database, as much of it is slow and expensive to gather.  This means the first search in a new location tends to be slow (a few seconds per plant), but later queries are fast.  Initial searches are also returned in the order ChatGPT returns them, but later searches may be sorted.

## TODO: Installation guide

## TODO: License info
