import React, { useState, useEffect } from "react";

// import material ui for zip code input
import TextField from "@mui/material/TextField";

// import reusable dropdown select component
import DropdownSelect from "./DropdownSelect";

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
    const sse = new EventSource(`${process.env.REACT_APP_URL_PREFIX}/plants?zip=${zip}&shade=${shade}&moisture=${moisture}`);

    sse.onmessage = e => {
        let plant = JSON.parse(e.data);
        setPlants((prevPlants) => [...prevPlants, plant]);
    };

    sse.addEventListener("close", (event) => {
        setLoading(false);
        sse.close()
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
                    console.log("Updating " + plant.scientific + " with url: " + imageUrl);
                    const updatedPlant = {
                        ...plant,
                        image_url: imageUrl
                    }

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
          setLoading(false);
          sse.close();
      };
        
    }, [formData]);

    const handleSubmit = async (event) => {
        event.preventDefault();
        setLoading(true);
        setFormData({
            zip: zip,
            shade: shade,
            moisture: moisture,
        });
    }

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <TextField id="zip" 
                   label="Zip Code" 
                   variant="outlined" 
                   onChange={handleZipChange} 
                   inputProps={{ inputMode: 'numeric', pattern: '[0-9]*' }} />
        <DropdownSelect id="shade" label="Shade" options={shadeOptions} onChange={handleShadeChange} value={shade}/>
        <DropdownSelect id="moisture" label="Moisture" options={moistureOptions} onChange={handleMoistureChange} value={moisture}/>

        <button type="submit">Find Native Plants</button>
      </form>
      <table>
        <tbody>
          {plants.map((plant, index) => (
            <tr>
              <td>
                <a href={plant.image_url}>
                  <img src={plant.image_url} alt={plant.image_url ? plant.common : null} width="150" />
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
      {loading ? (        
        <div className="spinner">
          <img src={`${process.env.PUBLIC_URL}/loading-earth.png`} pt="Loading" />
        </div>
      ) : null}
    </div>
  );
};

export default Plants;
