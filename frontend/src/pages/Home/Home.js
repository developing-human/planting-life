import { useState } from "react";
// components
import ConditionsForm from "../../components/ConditionsForm/ConditionsForm";
import IntroAccordion from "../../components/IntroAccordion/IntroAccordion";
import Spinner from "../../components/Spinner/Spinner";
import PlantCard from "../../components/PlantCard/PlantCard";
import Nursery from "../../components/Nursery/Nursery";

// material ui & styling
import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import "./Home.css";

import { Link, useNavigate } from "react-router-dom";

const Home = ({
  plants, setPlants, 
  nurseries, setNurseries, 
  selectedPlants, setSelectedPlants, 
  maxPlantsToDisplay, setMaxPlantsToDisplay,
  searchCriteria, setSearchCriteria
}) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [infoMessage, setInfoMessage] = useState(null);
  const [expanded, setExpanded] = useState('welcome');
  
  const showMoreButton = plants.length >= maxPlantsToDisplay;
  const showGardenButton = selectedPlants.length > 0;
  const showSpinner = loading && plants.length < maxPlantsToDisplay;
  const showSurvey = loading || plants.length > 0;

  const plantsWithImages = plants.filter((plant) => plant.image);

  const onMoreClick = () => {
    setMaxPlantsToDisplay((oldMax) => oldMax + 12);
  };

  const navigate = useNavigate();
  const onViewGardenClick = () => {
    console.log("hi mom", JSON.stringify(searchCriteria));
    fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        zipcode: searchCriteria.zip,
        shade: searchCriteria.shade,
        moisture: searchCriteria.moisture,
        plant_ids: selectedPlants.map((p) => p.id)
      })
    })
    .then(response => response.json())
    .then(data => {
      //TODO: Pass nurseries, search criteria, region_name, zipcode, read_id
      navigate(`gardens/${data.write_id}`, {state: {plants: selectedPlants}});
    })
    //TODO: Better error handling?
    .catch(error => console.error("Error:", error));
  };

  return (
    <>
      <ConditionsForm searchCriteria={searchCriteria}
                      setSearchCriteria={setSearchCriteria} 
                      setPlants={setPlants} 
                      setNurseries={setNurseries} 
                      setLoading={setLoading} 
                      setError={setError} 
                      setInfoMessage={setInfoMessage} 
                      setExpanded={setExpanded}
                      setMaxPlantsToDisplay={setMaxPlantsToDisplay}
                      setSelectedPlants={setSelectedPlants}
                      plants={plants}/>

      <div className="accordion-container">
        <IntroAccordion expanded={expanded} setExpanded={setExpanded}/>
      </div>

      {
        error || infoMessage ?
          <div className="alert-container">
            {error ? <Alert severity="error">{error}</Alert> : null}
            {infoMessage ? <Alert severity="info">{infoMessage}</Alert> : null}
          </div>
          : null
      }

      <div className="alert-container" id="top-survey-alert">
      {
        showSurvey ?
        <Alert severity="info">Help decide how Planting Life grows by <a href="https://docs.google.com/forms/d/e/1FAIpQLSfN9W9GusLRo5rIX3yENrBLKcNIu3y9BQpdRwOnCYYvTSX3zA/viewform?usp=sf_link" target="_blank" rel="noreferrer">sharing your thoughts</a>.</Alert>
        : null
      }
      </div>


      <section className="card-container">
        {plantsWithImages.slice(0, maxPlantsToDisplay).map((plant, index) => (
          <PlantCard plant={plant} key={index} setSelectedPlants={setSelectedPlants} setPlants={setPlants}/>
       ))}
        {showSpinner ? <Spinner /> : null}
        
      </section>
      
      <div className="button-container">
          {showMoreButton &&
              <Button className="more-button" 
                      type="submit" 
                      onClick={onMoreClick}>Load More</Button>
          }
      </div>
      <div className="button-container">
            <Link to="/gardens" state={{plants: selectedPlants}}>
                <Button className="garden-button" 
                        type="submit">
                  View Garden ({selectedPlants.length})
                </Button>
            </Link>

          {showGardenButton &&
            <Button className="garden-button" 
                    type="submit"
                    onClick={onViewGardenClick}>
              View Garden ({selectedPlants.length})
            </Button>
          }
      </div>


      {nurseries && nurseries.length > 0 && (plants.length >= 12 || !loading) ?
        <section className="card-container">
          <h1>Native Nurseries Near You</h1>
          {nurseries.map((nursery, index) => (
            <Nursery nursery={nursery} key={index} />
          ))}
        </section>
      : null}
    </>
  );
};

export default Home;
