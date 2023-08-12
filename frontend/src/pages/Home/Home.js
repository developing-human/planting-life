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

import { Link } from "react-router-dom";

const Home = () => {
  const [plants, setPlants] = useState(new Map());
  const [nurseries, setNurseries] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [infoMessage, setInfoMessage] = useState(null);
  const [expanded, setExpanded] = useState('welcome');
  const [maxPlantsToDisplay, setMaxPlantsToDisplay] = useState(12);
  const [selectedPlants, setSelectedPlants] = useState([]);

  const showMoreButton = plants.size >= maxPlantsToDisplay;
  const showGardenButton = selectedPlants.length > 0;
  const showSpinner = loading && plants.size < maxPlantsToDisplay;
  const showSurvey = loading || plants.size > 0;

  const plantsWithImages = Array.from(plants.values()).filter((plant) => plant.image);

  const onMoreClick = () => {
    setMaxPlantsToDisplay((oldMax) => oldMax + 12);
  };

  return (
    <>
      <ConditionsForm setPlants={setPlants} 
                      setNurseries={setNurseries} 
                      setLoading={setLoading} 
                      setError={setError} 
                      setInfoMessage={setInfoMessage} 
                      setExpanded={setExpanded}
                      setMaxPlantsToDisplay={setMaxPlantsToDisplay}
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
          <PlantCard plant={plant} key={index} setSelectedPlants={setSelectedPlants}/>
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
          {showGardenButton &&
              <Link to="/garden" state={{plants: selectedPlants}}>
                <Button className="garden-button" 
                        type="submit">
                  View Selected ({selectedPlants.length})
                </Button>
              </Link>
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
