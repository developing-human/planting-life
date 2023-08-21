import { useState, useRef, useEffect } from "react";

// components
import DropdownSelect from "./DropdownSelect";

// utilities
import sendRequest from "../utilities/plant-api";

// material ui
import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";
import Button from "@mui/material/Button";

// styling
import "./ConditionsForm.css";

function ConditionsForm({
  plants,
  setPlants,
  setNurseries,
  setLoading,
  setError,
  setInfoMessage,
  setExpanded,
  setMaxPlantsToDisplay,
  setLastSearchedCriteria,
  searchCriteria,
  setSearchCriteria,
  selectedPlants,
}) {
  // set drop down options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];
  const defaultShade = shadeOptions[1];
  const defaultMoisture = moistureOptions[1];

  const [eventSource, setEventSource] = useState(null);

  const plantsRef = useRef(plants);
  plantsRef.current = plants;

  const handleZipChange = (event) =>
    setSearchCriteria((prev) => {
      return { ...prev, zip: event.target.value };
    });

  const handleShadeChange = (newValue) =>
    setSearchCriteria((prev) => {
      return { ...prev, shade: newValue };
    });

  const handleMoistureChange = (newValue) =>
    setSearchCriteria((prev) => {
      return { ...prev, moisture: newValue };
    });

  // On page load, set the default values in the search criteria
  useEffect(() => {
    setSearchCriteria((prev) => {
      return {
        zip: prev.zip || "",
        shade: prev.shade || defaultShade,
        moisture: prev.moisture || defaultMoisture,
      };
    });
  }, [setSearchCriteria, defaultShade, defaultMoisture]);

  const handleSubmit = async (event) => {
    event.preventDefault();
    setExpanded(false);
    setPlants([]);
    setMaxPlantsToDisplay(12);
    setNurseries([]);
    setLoading(true);
    setError(null);
    setInfoMessage(null);

    // A brief delay on this helps it scroll nicely, since the accordion will
    // have collapsed.
    setTimeout(() => {
      // Find the top of the tab container
      const element = document.getElementById("top-survey-alert");
      const elementPosition = element.getBoundingClientRect().top;

      // If its negative, its above the top of the viewport and we need to scroll
      // up to the top when changing tabs.
      const offsetPosition = elementPosition + window.pageYOffset - 75;
      window.scrollTo({ top: offsetPosition, behavior: "auto" });
    }, 100);

    let formData = {
      zip: searchCriteria.zip,
      shade: searchCriteria.shade || defaultShade,
      moisture: searchCriteria.moisture || defaultMoisture,
    };

    setLastSearchedCriteria(formData);

    // Try to close an existing eventSource, loading behaves weird if
    // two EventSources are open at the same time.
    if (eventSource) {
      eventSource.close();
    }

    sendRequest(
      formData,
      setPlants,
      setLoading,
      setError,
      setInfoMessage,
      setEventSource,
      selectedPlants,
      () => {
        if (plantsRef.current.length === 0) {
          setInfoMessage(
            `Can't find anything near ${searchCriteria.zip} which thrives in ${searchCriteria.shade} and ${searchCriteria.moisture} moisture`
          );
        }
      }
    );

    // This loads at the same time as plants, but logic elsewhere hides the
    // nurseries until plants load enough for the screen to stop bouncing
    // around.
    fetch(
      `${process.env.REACT_APP_URL_PREFIX}/nurseries?zip=${searchCriteria.zip}`
    )
      .then((response) => response.json())
      .then((nurseries) => setNurseries(nurseries))
      .catch((error) => console.error("Error: ", error));
  };

  return (
    <form onSubmit={handleSubmit}>
      <Grid
        container
        spacing={3}
        style={{ display: "flex", justifyContent: "center" }}
      >
        <Grid item xs={12} sm={4}>
          <TextField
            id="zip"
            label="Zip Code"
            value={searchCriteria.zip}
            variant="outlined"
            onChange={handleZipChange}
            required
            sx={{ width: "100%" }}
            inputProps={{
              inputMode: "numeric",
              pattern: "[0-9]{5}",
              maxLength: 5,
              title: "US Zip Code",
            }}
            onKeyPress={(event) => {
              // Only allow numbers & Enter to be typed
              if (!/[0-9]/.test(event.key) && event.key !== "Enter") {
                event.preventDefault();
              }
            }}
          />
        </Grid>
        <Grid item xs={12} sm={4}>
          <DropdownSelect
            id="shade"
            label="Shade"
            options={shadeOptions}
            onChange={handleShadeChange}
            value={searchCriteria.shade || defaultShade}
          />
        </Grid>
        <Grid item xs={12} sm={4}>
          <DropdownSelect
            id="moisture"
            label="Moisture"
            options={moistureOptions}
            onChange={handleMoistureChange}
            value={searchCriteria.moisture || defaultMoisture}
          />
        </Grid>
        <Grid item>
          <Button type="submit">Find Native Plants</Button>
        </Grid>
      </Grid>
    </form>
  );
}

export default ConditionsForm;
