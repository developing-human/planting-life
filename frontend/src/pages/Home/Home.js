import { useState, useEffect, useCallback, useRef } from "react";
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

import { Link, useLocation, useNavigate } from "react-router-dom";

const Home = () => {
  console.log("In Home");
  const location = useLocation();
  const navigate = useNavigate();
  const containerRef = useRef();

  const state = window.history.state;
  const [plants, setPlants] = useState([]);
  const [nurseries, setNurseries] = useState([]);
  const [selectedPlants, setSelectedPlants] = useState([]);
  const [maxPlantsToDisplay, setMaxPlantsToDisplay] = useState(12);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [infoMessage, setInfoMessage] = useState(null);
  const [expanded, setExpanded] = useState('welcome');
  
  const handlePopState = useCallback(() => {
    console.log("In handlePopState");
    const state = window.history.state;
    console.log("state: " + JSON.stringify(state));
    if (state) {
      console.log("Initializing from history");

      console.log("Initializing plants: " + JSON.stringify(state.plants));
      setPlants(new Map(state.plants));

      console.log("Initializing nurseries: " + JSON.stringify(state.nurseries));
      setNurseries(state.nurseries || []);

      console.log("Initializing selectedPlants: " + JSON.stringify(state.selectedPlants));
      setSelectedPlants(state.selectedPlants || []);

      console.log("Initializing maxPlantsToDisplay: " + JSON.stringify(state.maxPlantsToDisplay));
      setMaxPlantsToDisplay(state.maxPlantsToDisplay);

      if (containerRef.current) {
        console.log("Trying to set scroll position: " + state.scrollPosition);
        containerRef.current.scrollTop = state.scrollPosition || 0;
      }
    }

  }, [setPlants, setNurseries, setSelectedPlants, setMaxPlantsToDisplay]);

  useEffect(() => {
    console.log("In useEffect");

    handlePopState();

    window.addEventListener('popstate', handlePopState);

    // Clean up the event listener when the component unmounts
    return () => {
      window.removeEventListener('popstate', handlePopState);
    };
  }, [handlePopState]);
    


  const showMoreButton = plants.size >= maxPlantsToDisplay;
  const showGardenButton = selectedPlants.length > 0;
  const showSpinner = loading && plants.size < maxPlantsToDisplay;
  const showSurvey = loading || plants.size > 0;

  const plantsWithImages = Array.from(plants.values()).filter((plant) => plant.image);

  const onMoreClick = () => {
    setMaxPlantsToDisplay((oldMax) => oldMax + 12);
  };

  const handleNavigateAway = () => {
    console.log("Navigating away... plants: " + JSON.stringify(plants.size));
    console.log("Navigating away... plants: " + JSON.stringify(plants));
    console.log("Navigating away... containerRef: " + (containerRef.current));
    const plantArray = Array.from(plants.entries());
    const stateToWrite = { 
      plants: plantArray, 
      nurseries: nurseries, 
      selectedPlants: selectedPlants, 
      maxPlantsToDisplay: maxPlantsToDisplay,
      scrollPosition: containerRef.current?.scrollTop || 0
    };
    console.log("Navigating away... state to write: " + JSON.stringify(stateToWrite));
    window.history.replaceState(stateToWrite, '');
    //window.history.replaceState({ ...window.history.state, ...{ plants: plants, nurseries: nurseries, selectedPlants: selectedPlants, maxPlantsToDisplay: maxPlantsToDisplay }}, '');
    console.log("window.history.state: " + JSON.stringify(window.history.state));
    navigate('/garden', { state: { selectedPlants} });
  }

  return (
    <div ref={containerRef} id="hi">
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
          {showGardenButton &&
                <Button className="garden-button" 
                        onClick={handleNavigateAway}
                        type="submit">
                  View Selected ({selectedPlants.length})
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
    </div>
  );
};

export default Home;
