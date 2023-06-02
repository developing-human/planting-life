export default async function sendRequest(formData, setPlants, setLoading, setError) {
    if (!formData) return;

    const { zip, shade, moisture } = formData;
    const sse = new EventSource(
      `${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`
    );

    sse.addEventListener("plant", (event) => {
      let plant = JSON.parse(event.data);
      console.log(plant);
      setPlants((prevPlants) => [...prevPlants, plant]);
    });

    // Hides the loading animation when the last plant appears,
    // rather than when all plants finish loading.
    sse.addEventListener("allPlantsLoaded", (event) => {
      setLoading(false);
    });

    sse.addEventListener("close", (event) => {
      setLoading(false);
      sse.close();
    });

    sse.addEventListener("error", (event) => {
      setLoading(false);
      setError("Well that's embarassing... please try again.");
      sse.close();
    });

    sse.addEventListener("image", (event) => {
      // get JSON image data
      const image = JSON.parse(event.data);

      // grab image scientific name to compare to AI data
      const scientificName = image.scientificName;

      // grab relevant image and attribution data
      const imageUrl = image.thumbnailUrl;
      const cardUrl = image.cardUrl;
      const originalUrl = image.originalUrl;
      const author = image.author;
      const title = image.title;
      const license = image.license;
      const licenseUrl = image.licenseUrl;

      setPlants((prevPlants) => {
        const newPlants = prevPlants.map((plant) => {
          if (plant.scientific === scientificName) {
            const updatedPlant = {
              ...plant,
              image_url: imageUrl,
              card_url: cardUrl,
              original_url: originalUrl,
              title: title,
              author: author,
              license: license,
              licenseUrl: licenseUrl,
            };

            return updatedPlant;
          }

          return plant;
        });

        return newPlants;
      });
    });

    sse.addEventListener("descriptionDelta", (event) => {
      // get JSON image data
      const payload = JSON.parse(event.data);
      setPlants((prevPlants) => {
        const newPlants = prevPlants.map((plant) => {
          if (plant.scientific === payload.scientificName) {
            const delta = payload.descriptionDelta;
            const updatedPlant = {
              ...plant,
              description: plant.description
                ? plant.description + delta
                : delta,
            };

            return updatedPlant;
          }

          return plant;
        });

        return newPlants;
      });
    });

    return () => {
      sse.close();
    };
}

export async function getData(plants, loading, alert) {
    let data = {plants, loading, alert}
    return data;
}