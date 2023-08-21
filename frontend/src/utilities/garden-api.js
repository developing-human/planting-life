export async function getGarden(id, setGarden, onError) {
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
      console.log(garden);
      console.log("Calling setGarden");
      setGarden(garden);
    })
    .catch((error) => {
      onError(error);
    });

  return () => {};
}

export async function putGarden(garden) {
  return () => {};
}

export async function post(garden) {
  return () => {};
}
