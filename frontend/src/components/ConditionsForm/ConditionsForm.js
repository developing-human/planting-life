import { useEffect, useState } from "react";

// material ui
import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";
import Button from "@mui/material/Button";

// components
import DropdownSelect from "../DropdownSelect/DropdownSelect";

// utilities
import sendRequest from "../../utilities/openai-api";

// other
import Alert from "@mui/material/Alert";
import PlantCard from "../PlantCard/PlantCard";
import Spinner from "../Spinner/Spinner";

function ConditionsForm() {
  // set drop down options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];
  const defaultShade = shadeOptions[1];
  const defaultMoisture = moistureOptions[1];

  const [formData, setFormData] = useState(null);
  const [plants, setPlants] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
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
    sendRequest(formData, setPlants, setLoading, setError)
  }, [formData]);

  const handleSubmit = async (event) => {
    event.preventDefault();
    setPlants([]);
    setLoading(true);
    setError(null);
    setFormData({
      zip: zip,
      shade: shade,
      moisture: moisture,
    });
  };

  return (
    <div>
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
      
      {error ? <Alert severity="error">{error}</Alert> : null}

      <section id="returned-plants">
        {plants.map((plant, index) => (
          <PlantCard plant={plant} key={index} />
        ))}

        {loading ? <Spinner /> : null}
      </section>
    </div>
  );
}

export default ConditionsForm;
