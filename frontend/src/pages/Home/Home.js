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

const Home = () => {
  const [plants, setPlants] = useState([]);
  const [nurseries, setNurseries] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [infoMessage, setInfoMessage] = useState(null);
  const [expanded, setExpanded] = useState('welcome');
  const [maxPlantsToDisplay, setMaxPlantsToDisplay] = useState(12);

  const onMoreClick = () => {
    setMaxPlantsToDisplay((oldMax) => {
      const newMax = oldMax + 12;

      // This is a little non-obvious, but since we hide cards which lack
      // images, it was possible that increasing the max by 12 wouldn't
      // show 12 more.  This counts how many of the new plants are missing
      // images, and adjusts the count a bit higher to ensure we always
      // show multiples of 12.
      const newPlants = plants.slice(oldMax, newMax);
      const plantsWithoutImages = newPlants.filter(plant => !plant.image);
      const numPlantsWithoutImages = plantsWithoutImages.length;

      return newMax + numPlantsWithoutImages;
    });
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

      <div className="accordion-container"><IntroAccordion expanded={expanded} setExpanded={setExpanded}/></div>

      <div className="alert-container">
        {error ? <Alert severity="error">{error}</Alert> : null}
        {infoMessage ? <Alert severity="info">{infoMessage}</Alert> : null}
      </div>

      <section className="card-container">
        {plants.slice(0, maxPlantsToDisplay).map((plant, index) => (
          plant.image ? <PlantCard plant={plant} key={index} /> 
            : null
        ))}

        
        {loading && plants.length < maxPlantsToDisplay ? <Spinner /> : null}
        
      </section>
      
      <div className="more-container">
        {plants.length >= maxPlantsToDisplay ?
           <Button type="submit" onClick={onMoreClick}>Load More</Button> : null}
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
