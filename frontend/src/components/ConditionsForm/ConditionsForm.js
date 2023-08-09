import { useState, useRef } from "react";

// components
import DropdownSelect from "../DropdownSelect/DropdownSelect";

// utilities
import sendRequest from "../../utilities/plant-api";

// material ui
import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";
import Button from "@mui/material/Button";

// styling
import "./ConditionsForm.css"

function ConditionsForm({ setPlants, setNurseries, setLoading, setError, setInfoMessage, setExpanded, plants, setMaxPlantsToDisplay }) {
  // set drop down options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];
  const defaultShade = shadeOptions[1];
  const defaultMoisture = moistureOptions[1];

  const [zip, setZip] = useState("");
  const [shade, setShade] = useState(defaultShade);
  const [moisture, setMoisture] = useState(defaultMoisture);
  const [eventSource, setEventSource] = useState(null);

  const plantsRef = useRef(plants);
  plantsRef.current = plants;

  const handleZipChange = (event) => {
    setZip(event.target.value);
  };

  const handleShadeChange = (newValue) => {
    setShade(newValue);
  };

  const handleMoistureChange = (newValue) => {
    setMoisture(newValue);
  };

  const handleSubmit = async (event) => {
    event.preventDefault();
    setExpanded(false);
    setPlants(new Map());
    setMaxPlantsToDisplay(12);
    setNurseries([]);
    setLoading(true);
    setError(null);
    setInfoMessage(null);

    // A brief delay on this helps it scroll nicely, since the accordion will
    // have collapsed.
    setTimeout(() => {
      document.getElementById("top-survey-alert").scrollIntoView({behavior: 'smooth'});
    }, 100);

    let formData = {
      zip: zip,
      shade: shade,
      moisture: moisture,
    };

    // Try to close an existing eventSource, loading behaves weird if
    // two EventSources are open at the same time.
    if (eventSource) {
      eventSource.close();
    }

    sendRequest(formData, setPlants, setLoading, setError, setInfoMessage, setEventSource, () => {
      if (plantsRef.current.length === 0) {
        setInfoMessage(`Can't find anything near ${zip} which thrives in ${shade} and ${moisture} moisture`);
      }
    });

    // This loads at the same time as plants, but logic elsewhere hides the 
    // nurseries until plants load enough for the screen to stop bouncing
    // around.
    fetch(`${process.env.REACT_APP_URL_PREFIX}/nurseries?zip=${formData.zip}`)
      .then(response => response.json())
      .then(nurseries => setNurseries(nurseries))
      .catch(error => console.error('Error: ', error));
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
                if (!/[0-9]/.test(event.key) && event.key !== 'Enter') {
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
              value={shade}
            />
          </Grid>
          <Grid item xs={12} sm={4}>
            <DropdownSelect
              id="moisture"
              label="Moisture"
              options={moistureOptions}
              onChange={handleMoistureChange}
              value={moisture}
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
