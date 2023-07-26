import { useState } from "react";

// components
import ConditionsForm from "../../components/ConditionsForm/ConditionsForm";
import IntroAccordion from "../../components/IntroAccordion/IntroAccordion";
import Spinner from "../../components/Spinner/Spinner";
import PlantCard from "../../components/PlantCard/PlantCard";
import Nursery from "../../components/Nursery/Nursery";

// material ui & styling
import Alert from "@mui/material/Alert";
import "./Home.css";

const Home = () => {
  const [plants, setPlants] = useState([]);
  const [nurseries, setNurseries] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [expanded, setExpanded] = useState('welcome');

  return (
    <>
      <ConditionsForm setPlants={setPlants} 
                      setNurseries={setNurseries} 
                      setLoading={setLoading} 
                      setError={setError} 
                      setExpanded={setExpanded}/>

      <div className="accordion-container"><IntroAccordion expanded={expanded} setExpanded={setExpanded}/></div>

      {error ? <Alert severity="error">{error}</Alert> : null}

      <section className="card-container">
        {plants.map((plant, index) => (
          plant.image ? <PlantCard plant={plant} key={index} /> 
            : null
        ))}

        {loading ? <Spinner /> : null}
      </section>

      {nurseries && nurseries.length > 0 ?
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
