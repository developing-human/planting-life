import { useEffect, useState } from "react";

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

function ConditionsForm({ setPlants, setNurseries, setLoading, setError, setExpanded }) {
  // set drop down options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];
  const defaultShade = shadeOptions[1];
  const defaultMoisture = moistureOptions[1];

  const [formData, setFormData] = useState(null);
  const [zip, setZip] = useState("");
  const [shade, setShade] = useState(defaultShade);
  const [moisture, setMoisture] = useState(defaultMoisture);

  const handleZipChange = (event) => {
    setZip(event.target.value);
  };

  const handleShadeChange = (newValue) => {
    setShade(newValue);
  };

  const handleMoistureChange = (newValue) => {
    setMoisture(newValue);
  };

  useEffect(() => {
    if (!formData) {
      return;
    }

    sendRequest(formData, setPlants, setLoading, setError, () => {

      // This is a callback to make the nurseries not load until after
      // plants are loaded.  Loading it first made it distracting and
      // impossible to read as the cards were loading.
      fetch(`${process.env.REACT_APP_URL_PREFIX}/nurseries?zip=${formData.zip}`)
        .then(response => response.json())
        .then(nurseries => setNurseries(nurseries))
        .catch(error => console.error('Error: ', error));
    });

  }, [formData, setPlants, setNurseries, setLoading, setError]);

  const handleSubmit = async (event) => {
    event.preventDefault();
    setExpanded(false);
    setPlants([]);
    setNurseries([]);
    setLoading(true);
    setError(null);
    setFormData({
      zip: zip,
      shade: shade,
      moisture: moisture,
    });
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
              sx={{ width: "100%" }}
              inputProps={{ inputMode: "numeric", pattern: "[0-9]*" }}
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
