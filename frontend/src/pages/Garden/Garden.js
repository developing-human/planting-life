// components
import PlantCard from "../../components/PlantCard/PlantCard";

import "./Garden.css";

import { useLocation } from 'react-router-dom';


const Garden = () => {
  let { state } = useLocation();

  return (
    <>
      <section className="card-container">
        {state.plants.map((plant, index) => (
          plant.image ? <PlantCard plant={plant} key={index} showAddButton={false}/> 
            : null
        ))}
      </section>
    </>
  );
};

export default Garden;
