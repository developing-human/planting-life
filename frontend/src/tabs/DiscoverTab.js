import { useEffect, useState, useRef } from "react";

// components
import ConditionsForm from "../components/ConditionsForm";
import Spinner from "../components/Spinner";
import PlantCard from "../components/PlantCard";

// material ui & styling
import Alert from "@mui/material/Alert";

const DiscoverTab = ({
  plants,
  setPlants,
  setNurseries,
  garden,
  setGarden,
  searchCriteria,
  setSearchCriteria,
  setLastSearchedCriteria,
  error,
  setError,
  eventSource,
  setEventSource,
}) => {
  const [infoMessage, setInfoMessage] = useState(null);
  const [loading, setLoading] = useState(false);
  const [maxPlantsToDisplay, setMaxPlantsToDisplay] = useState(12);
  const lastPlantRef = useRef(null);

  const showSpinner = loading && plants.length < maxPlantsToDisplay;
  const showSurvey = loading || plants.length > 0;

  const plantsWithImages = plants.filter((plant) => plant.image);

  // When plants or maxPlantsToDisplay changes, watch for when the last plant
  // comes into view.  When it does, increment the # of plants to display.
  // This lets us load all the JSON at once (cheap) while waiting to load images
  // (expensive) until they're needed.
  useEffect(() => {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        setMaxPlantsToDisplay((oldMax) => oldMax + 12);
      }
    });

    if (lastPlantRef.current) {
      observer.observe(lastPlantRef.current);
    }

    return () => observer.disconnect();
  }, [plants, maxPlantsToDisplay]);

  return (
    <>
      <ConditionsForm
        setLastSearchedCriteria={setLastSearchedCriteria}
        setPlants={setPlants}
        setNurseries={setNurseries}
        setLoading={setLoading}
        setError={setError}
        setInfoMessage={setInfoMessage}
        setMaxPlantsToDisplay={setMaxPlantsToDisplay}
        plants={plants}
        searchCriteria={searchCriteria}
        setSearchCriteria={setSearchCriteria}
        selectedPlants={garden.plants}
        eventSource={eventSource}
        setEventSource={setEventSource}
      />

      {error || infoMessage ? (
        <div className="alert-container">
          {error ? <Alert severity="error">{error}</Alert> : null}
          {infoMessage ? <Alert severity="info">{infoMessage}</Alert> : null}
        </div>
      ) : null}

      <div className="alert-container" id="top-survey-alert">
        {showSurvey ? (
          <Alert severity="info">
            Help decide how Planting Life grows by{" "}
            <a
              href="https://docs.google.com/forms/d/e/1FAIpQLSfN9W9GusLRo5rIX3yENrBLKcNIu3y9BQpdRwOnCYYvTSX3zA/viewform?usp=sf_link"
              target="_blank"
              rel="noreferrer"
            >
              sharing your thoughts
            </a>
            .
          </Alert>
        ) : null}
      </div>

      <section className="card-container" id="discover-cards">
        {plantsWithImages.slice(0, maxPlantsToDisplay).map((plant, index) => (
          <div
            key={plant.id}
            ref={index + 1 === maxPlantsToDisplay ? lastPlantRef : null}
          >
            <PlantCard
              plant={plant}
              setGarden={setGarden}
              setPlants={setPlants}
            />
          </div>
        ))}
        {showSpinner ? <Spinner /> : null}
      </section>
    </>
  );
};

export default DiscoverTab;
