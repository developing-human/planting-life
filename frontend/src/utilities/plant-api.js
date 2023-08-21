export default async function sendRequest(
  formData,
  setPlants,
  setLoading,
  setError,
  setInfoMessage,
  setEventSource,
  selectedPlants,
  onFinishedLoading
) {
  const { zip, shade, moisture } = formData;
  const sse = new EventSource(
    `${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`
  );

  setEventSource(sse);

  sse.addEventListener("plant", (event) => {
    let newPlant = JSON.parse(event.data);
    newPlant.selected = selectedPlants.some(
      (sp) => sp.scientific === newPlant.scientific
    );

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
    setLoading(false);
    onFinishedLoading();
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

  return () => {
    sse.close();
  };
}
