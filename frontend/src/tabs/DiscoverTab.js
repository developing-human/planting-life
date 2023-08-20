import { useState } from "react";

// components
import ConditionsForm from "../components/ConditionsForm";
import IntroAccordion from "../components/IntroAccordion";
import Spinner from "../components/Spinner";
import PlantCard from "../components/PlantCard";

// material ui & styling
import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";

const DiscoverTab = ({
  plants,
  setPlants,
  setNurseries,
  setSelectedPlants,
  searchCriteria,
  setSearchCriteria,
  setLastSearchedCriteria,
}) => {
  const [error, setError] = useState(null);
  const [infoMessage, setInfoMessage] = useState(null);
  const [expanded, setExpanded] = useState("welcome");
  const [loading, setLoading] = useState(false);
  const [maxPlantsToDisplay, setMaxPlantsToDisplay] = useState(12);

  const showMoreButton = plants.length >= maxPlantsToDisplay;
  const showSpinner = loading && plants.length < maxPlantsToDisplay;
  const showSurvey = loading || plants.length > 0;

  const plantsWithImages = plants.filter((plant) => plant.image);

  const onMoreClick = () => {
    setMaxPlantsToDisplay((oldMax) => oldMax + 12);
  };

  return (
    <>
      <ConditionsForm
        setLastSearchedCriteria={setLastSearchedCriteria}
        setPlants={setPlants}
        setNurseries={setNurseries}
        setLoading={setLoading}
        setError={setError}
        setInfoMessage={setInfoMessage}
        setExpanded={setExpanded}
        setMaxPlantsToDisplay={setMaxPlantsToDisplay}
        setSelectedPlants={setSelectedPlants}
        plants={plants}
        searchCriteria={searchCriteria}
        setSearchCriteria={setSearchCriteria}
      />

      <div className="accordion-container">
        <IntroAccordion expanded={expanded} setExpanded={setExpanded} />
      </div>

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
          <PlantCard
            plant={plant}
            key={plant.id}
            setSelectedPlants={setSelectedPlants}
            setPlants={setPlants}
          />
        ))}
        {showSpinner ? <Spinner /> : null}
      </section>

      <div className="button-container">
        {showMoreButton && (
          <Button className="more-button" type="submit" onClick={onMoreClick}>
            Load More
          </Button>
        )}
      </div>
    </>
  );
};

export default DiscoverTab;
