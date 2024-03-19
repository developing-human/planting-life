export async function getPlants(searchParams, setPlants, setLoading) {
  const { zip, shade, moisture } = searchParams;
  fetch(`${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`)
    .then((response) => response.json())
    .then((plants) => {
      setPlants(plants);
      setLoading(false);
    })
    .catch((error) => console.error("Error: ", error));

  return () => { };
}

export default async function openPlantsStream(
  searchParams,
  setPlants,
  setError,
  setEventSource,
  selectedPlants,
  setLoading
) {
  const { zip, shade, moisture, scientificName } = searchParams;
  let sse = null;
  let queryingByName = false;
  if (zip && shade && moisture) {
    sse = new EventSource(
      `${process.env.REACT_APP_URL_PREFIX}/plants/stream?zip=${zip}&shade=${shade}&moisture=${moisture}`
    );
  } else {
    queryingByName = true;
    sse = new EventSource(
      `${process.env.REACT_APP_URL_PREFIX}/plants/stream/${scientificName}`
    );
  }

  if (setEventSource) {
    setEventSource(sse);
  }

  sse.addEventListener("plant", (event) => {
    let newPlant = JSON.parse(event.data);

    // When querying by name, assume it is selected.  Because this use case is
    // when searching for plants by name via garden screen.
    newPlant.selected =
      queryingByName ||
      selectedPlants.some((sp) => sp.scientific === newPlant.scientific);

    setPlants((prevPlants) => {
      const index = prevPlants.findIndex(
        (p) => p.scientific === newPlant.scientific
      );
      const newPlants = prevPlants.slice();
      if (index === -1) {
        newPlants.push(newPlant);
      } else {
        newPlants[index] = { ...prevPlants[index], ...newPlant };
      }

      return newPlants;
    });
  });

  // Hides the loading animation when the last plant appears,
  // rather than when all plants finish loading.
  sse.addEventListener("allPlantsLoaded", (event) => {
    if (setLoading) {
      setLoading(false);
    }
  });

  sse.addEventListener("close", (event) => {
    if (setLoading) {
      setLoading(false);
    }
    sse.close();
  });

  sse.addEventListener("error", (event) => {
    if (setLoading) {
      setLoading(false);
    }
    setError("Well that's embarassing... please try again.");
    sse.close();
  });

  return () => {
    sse.close();
  };
}

export async function fetchPlantsByName(name, onSuccess, onError) {
  return fetch(`${process.env.REACT_APP_URL_PREFIX}/plants?name=${name}`, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
    },
  }).then((response) => {
    if (!response.ok) {
      throw new Error(
        `Error fetching plants by name, status: ${response.status}`
      );
    }
    return response.json();
  });
}
