export async function getGarden(id, onSuccess, onError) {
  console.log(onError);
  fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens/${id}`, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
    },
  })
    .then((response) => {
      if (!response.ok) {
        throw new Error(`Error fetching /gardens, status: ${response.status}`);
      }

      return response.json();
    })
    .then((garden) => {
      onSuccess(garden);
    })
    .catch((error) => {
      onError(error);
    });

  return () => {};
}

export async function saveGarden(
  garden,
  setGarden,
  lastSearchedCriteria,
  onError
) {
  if (!garden.write_id) {
    postGarden(garden, setGarden, lastSearchedCriteria, onError);
  } else {
    putGarden(garden, onError);
  }
}

async function postGarden(garden, setGarden, lastSearchedCriteria, onError) {
  fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      zipcode: lastSearchedCriteria.zip,
      shade: lastSearchedCriteria.shade,
      moisture: lastSearchedCriteria.moisture,
      plant_ids: garden.plants.map((p) => p.id),
      name: garden.name,
    }),
  })
    .then((response) => response.json())
    .then((data) => {
      setGarden((prevGarden) => {
        return { ...prevGarden, ...data };
      });
    })
    .catch((error) => {
      onError(error);
    });

  return () => {};
}

async function putGarden(garden, onError) {
  fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens/${garden.write_id}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      plant_ids: garden.plants.map((p) => p.id),
      name: garden.name,
    }),
  })
    .then((response) => response.json())
    .catch((error) => {
      onError(error);
    });

  return () => {};
}
