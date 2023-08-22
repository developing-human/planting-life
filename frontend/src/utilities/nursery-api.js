export async function getNurseries(zip, setNurseries) {
  fetch(`${process.env.REACT_APP_URL_PREFIX}/nurseries?zip=${zip}`)
    .then((response) => response.json())
    .then((nurseries) => setNurseries(nurseries))
    .catch((error) => console.error("Error: ", error));

  return () => {};
}
