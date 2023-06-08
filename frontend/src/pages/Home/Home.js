import { useState } from "react";

// components
import ConditionsForm from "../../components/ConditionsForm/ConditionsForm";
import Accordion from "../../components/IntroAccordion/IntroAccordion";
import Spinner from "../../components/Spinner/Spinner";
import PlantCard from "../../components/PlantCard/PlantCard";

// material ui & styling
import Alert from "@mui/material/Alert";
import "./Home.css";

const Home = () => {
  const [plants, setPlants] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [expanded, setExpanded] = useState('welcome');

  return (
    <>
      <ConditionsForm setPlants={setPlants} setLoading={setLoading} setError={setError} setExpanded={setExpanded}/>

      <div className="accordion-container"><Accordion expanded={expanded} setExpanded={setExpanded}/></div>

      {error ? <Alert severity="error">{error}</Alert> : null}

      <section id="returned-plants">
        {plants.map((plant, index) => (
          <PlantCard plant={plant} key={index} />
        ))}

        {loading ? <Spinner /> : null}
      </section>
    </>
  );
};

export default Home;
