export async function getPlants(searchParams, setPlants, setError, selectedPlants, setLoading) {
  const { zip, shade, moisture } = searchParams;
  fetch(`${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`)
    .then((response) => response.json())
    .then((plants) => {
      if (selectedPlants !== undefined) {
        for (const plant of plants) {
          plant.selected = selectedPlants.some((sp) => sp.id === plant.id);
        }
      }
      setPlants(plants);
    })
    .catch((error) => {
      console.error("Error: ", error);
      setError("Well that's embarassing... please try again.");
    })
    .finally(() => {
      if (setLoading) {
        setLoading(false);
      }
    });

  return () => { };
}

export async function getPlant(id, onSuccess) {
  fetch(`${process.env.REACT_APP_URL_PREFIX}/plants/${id}`)
    .then((response) => response.json())
    .then(onSuccess)
    .catch((error) => {
      console.error("Error: ", error);
    });

  return () => { };
}

//TODO: Can getPlants take fewer params?  Maybe an onSuccess instead?
//TODO: Cleanup refs to eventSource
export async function fetchPlantsByName(name) {
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
