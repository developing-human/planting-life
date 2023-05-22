import React, { useState, useEffect } from "react";

// import material ui for zip code input
import TextField from "@mui/material/TextField";

// import reusable dropdown select & attribution components
import DropdownSelect from "./DropdownSelect";
import AttributionPopover from "./AttributionPopover";

import "./Plants.css";

const Plants = () => {
  const [plants, setPlants] = useState([]);
  const [formData, setFormData] = useState(null);
  const [zip, setZip] = useState("");
  const [shade, setShade] = useState("");
  const [moisture, setMoisture] = useState("");

  const handleZipChange = (event) => {
    setZip(event.target.value);
  };

  const handleShadeChange = (newValue) => {
    setShade(newValue);
  };

  const handleMoistureChange = (newValue) => {
    setMoisture(newValue);
  };

  // set drop down menu options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];

  useEffect(() => {
    if (!formData) return;

    const { zip, shade, moisture } = formData;
    const sse = new EventSource(
      `${process.env.REACT_APP_URL_PREFIX}/plants_mock?zip=${zip}&shade=${shade}&moisture=${moisture}`
    );

    sse.onmessage = (e) => {
      let plant = JSON.parse(e.data);
      setPlants((prevPlants) => [...prevPlants, plant]);
    };

    sse.addEventListener("close", (event) => {
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
        console.log("About to update url, plants.length=" + prevPlants.length);
        const newPlants = prevPlants.map((plant) => {
          if (plant.scientific) {
          // if (plant.scientific === scientificName) {
            console.log(
              "Updating " + plant.scientific + " with url: " + imageUrl
            );
            const updatedPlant = {
              ...plant,
              image_url: imageUrl,
              original_url: originalUrl,
              title: title,
              author: author,
              license: license,
              licenseUrl: licenseUrl,
            };

            console.log("Updated plant");
            return updatedPlant;
          }

          console.log("Not updating plant, but keeping it");
          return plant;
        });

        console.log("Updating newPlants.length=" + prevPlants.length);

        return newPlants;
      });
    });

    return () => {
      sse.close();
    };
  }, [formData]);

  const handleSubmit = async (event) => {
    event.preventDefault();
    setFormData({
      zip: zip,
      shade: shade,
      moisture: moisture,
    });
  };

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <TextField
          id="zip"
          label="Zip Code"
          variant="outlined"
          onChange={handleZipChange}
          inputProps={{ inputMode: "numeric", pattern: "[0-9]*" }}
        />
        <DropdownSelect
          id="shade"
          label="Shade"
          options={shadeOptions}
          onChange={handleShadeChange}
        />
        <DropdownSelect
          id="moisture"
          label="Moisture"
          options={moistureOptions}
          onChange={handleMoistureChange}
        />

        <button type="submit">Find Native Plants</button>
      </form>
      <table>
        <tbody>
          {plants.map((plant, index) => (
            <tr>
              <td>
                <a class="plantImageContainer" href={plant.original_url} target="blank">
                  <img class="plantImage" src={plant.image_url} alt={plant.common} />
                </a>
                <figcaption><AttributionPopover caption={`© Photo by ${plant.author}, click for details.`} title={plant.title} author={plant.author} license={plant.license}/></figcaption>
              </td>
              <td>
                <b>{plant.common}</b>
                <i>{plant.scientific}</i>
                <br /> <br />
                Blooms in {plant.bloom.toLowerCase()}. {plant.description}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default Plants;
