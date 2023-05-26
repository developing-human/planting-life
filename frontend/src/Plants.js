import React, { useState, useEffect } from "react";

// import material ui for zip code input
import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";

// import reusable dropdown select & attribution components
import DropdownSelect from "./DropdownSelect";
import AttributionPopover from "./AttributionPopover";

import "./Plants.css";
import "./spinner.css";

const Plants = () => {
  // set drop down menu options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];

  const defaultShade = shadeOptions[1];
  const defaultMoisture = moistureOptions[1];

  const [plants, setPlants] = useState([]);
  const [formData, setFormData] = useState(null);
  const [zip, setZip] = useState("");
  const [shade, setShade] = useState(defaultShade);
  const [moisture, setMoisture] = useState(defaultMoisture);
  const [loading, setLoading] = useState(false);

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
    if (!formData) return;

    const { zip, shade, moisture } = formData;
    const sse = new EventSource(
      `${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`
    );

    sse.onmessage = (e) => {
      let plant = JSON.parse(e.data);
      setPlants((prevPlants) => [...prevPlants, plant] );
    };

    sse.addEventListener("close", (event) => {
        setLoading(false);
        sse.close();
    });

    sse.addEventListener("image", (event) => {
      // get JSON image data
      const image = JSON.parse(event.data);

      // grab image scientific name to compare to AI data
      const scientificName = image.scientificName;

      // grab relevant image and attribution data
      const imageUrl = image.thumbnailUrl;
      const originalUrl = image.originalUrl;
      const author = image.author;
      const title = image.title;
      const license = image.license;
      const licenseUrl = image.licenseUrl;

      setPlants((prevPlants) => {
        const newPlants = prevPlants.map((plant) => {
          if (plant.scientific === scientificName) {
            const updatedPlant = {
              ...plant,
              image_url: imageUrl,
              original_url: originalUrl,
              title: title,
              author: author,
              license: license,
              licenseUrl: licenseUrl,
            };

            return updatedPlant;
          }

          return plant;
        });


        return newPlants;
      });
    });

    sse.addEventListener("descriptionDelta", (event) => {
      // get JSON image data
      const payload = JSON.parse(event.data);
      setPlants((prevPlants) => {
        const newPlants = prevPlants.map((plant) => {
          if (plant.scientific === payload.scientificName) {
            const delta = payload.descriptionDelta;
            const updatedPlant = {
              ...plant,
              description: plant.description ? plant.description + delta : delta
            };

            return updatedPlant;
          }

          return plant;
        });


        return newPlants;
      });
    });

    return () => {
      sse.close();
    };
  }, [formData]);

  const handleSubmit = async (event) => {
    event.preventDefault();
    setPlants([]);
    setLoading(true);
    setFormData({
      zip: zip,
      shade: shade,
      moisture: moisture,
    });
  };


  return (
    <div>
      <form onSubmit={handleSubmit}>

        <Grid container spacing={3} style={{display: 'flex'}}>
          <Grid item xs={12} sm={4}>
            <TextField 
              id="zip" 
              label="Zip Code" 
              variant="outlined" 
              onChange={handleZipChange} 
              sx={{width: '100%'}}
              inputProps={{ inputMode: 'numeric', pattern: '[0-9]*' }} />
          </Grid>
          <Grid item xs={12} sm={4}>
            <DropdownSelect 
              id="shade"
              label="Shade"
              options={shadeOptions}
              onChange={handleShadeChange}
              value={shade}/>
          </Grid>
          <Grid item xs={12} sm={4}>
            <DropdownSelect 
              id="moisture"
              label="Moisture"
              options={moistureOptions}
              onChange={handleMoistureChange}
              value={moisture}/>
          </Grid>
          <Grid item xs={12} sm={12}>
            <button type="submit">Find Native Plants</button>
          </Grid>
        </Grid>
      </form>
      <table id="returned-plants">
        <tbody>
          {plants.map((plant, index) => (
              <tr>
                <td className="imageCell">
                  <a href={plant.original_url} target="_blank" rel="noreferrer">
                    <img className="plantImage" src={plant.image_url} alt={plant.image_url ? plant.common : null} />
                  </a>
                  {
                    plant.author ? (
                      <figcaption>
                        <AttributionPopover
                          caption={`© Photo by ${plant.author}`}
                          title={plant.title}
                          author={plant.author}
                          license={plant.license}
                          link={plant.licenseUrl}/></figcaption>
                    ) : null
                  }
                </td>
                <td>
                  <b>{plant.common}</b>
                  <i>{plant.scientific}</i>
                  <br /> <br />
                  Blooms in {plant.bloom.toLowerCase()}. {plant.description}
                </td>
              </tr>
              )
          )}
        </tbody>
      </table>
      {loading ? (        
        <div className="spinner">
          <img src={`${process.env.PUBLIC_URL}/loading-earth.png`} alt="Loading" />
        </div>
      ) : null}
    </div>
  );
};

export default Plants;
