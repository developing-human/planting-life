export default async function sendRequest(formData, setPlants, setLoading, setError, setInfoMessage, setEventSource, onFinishedLoading) {
    const { zip, shade, moisture } = formData;
    const sse = new EventSource(
      `${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`
    );

    setEventSource(sse);

    sse.addEventListener("plant", (event) => {
      let newPlant = JSON.parse(event.data);
      setPlants((prevPlants) => {
      
        const existing = prevPlants.get(newPlant.scientific);
        const newPlants = new Map(prevPlants);
        if (existing === undefined) {
          newPlants.set(newPlant.scientific, newPlant);
        } else {
          newPlants.set(newPlant.scientific, {...existing, ...newPlant });
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
