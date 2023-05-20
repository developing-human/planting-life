import React, { useState, useEffect } from "react";

// import material ui for zip code input
import TextField from "@mui/material/TextField";

// import reusable dropdown select component
import DropdownSelect from "./DropdownSelect";

import "./Plants.css";

const Plants = () => {
  const [plants, setPlants] = useState([]);
  const [formData, setFormData] = useState(null);

  // set drop down menu options
  const shadeOptions = ["Full Shade", "Partial Shade", "Full Sun"];
  const moistureOptions = ["Low", "Medium", "High"];

  useEffect(() => {
    if (!formData) return;

    const { zip, shade, moisture } = formData;
    const sse = new EventSource(`${process.env.REACT_APP_API_URL}/plants_mock?zip=${zip}&shade=${shade}&moisture=${moisture}`);

    sse.onmessage = (e) => {
      let plant = JSON.parse(e.data);
      setPlants((prevPlants) => [...prevPlants, plant]);
    };

    sse.addEventListener("close", (event) => {
      sse.close();
    });

    sse.addEventListener("image_url", (event) => {
      console.log(event.data);
      const splitData = event.data.split("::");
      const scientificName = splitData[0];
      const imageUrl = splitData[1];

      setPlants((prevPlants) => {
        console.log("About to update url, plants.length=" + prevPlants.length);
        const newPlants = prevPlants.map((plant) => {
          if (plant.scientific === scientificName) {
            console.log(
              "Updating " + plant.scientific + " with url: " + imageUrl
            );
            const updatedPlant = {
              ...plant,
              image_url: imageUrl,
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
      zip: document.getElementById("zip").value,
      shade: document.getElementById("shade"),
      moisture: document.getElementById("moisture"),
    });
  };

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <TextField id="zip" label="Zip Code" variant="outlined" />

        <DropdownSelect id="shade" label="Shade" options={shadeOptions} />

        <DropdownSelect id="moisture" label="Moisture" options={moistureOptions} />

        <button type="submit">Find Native Plants</button>
      </form>
      <table>
        <tbody>
          {plants.map((plant, index) => (
            <tr>
              <td>
                <a href={plant.image_url}>
                  <img src={plant.image_url} width="150" />
                </a>
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
