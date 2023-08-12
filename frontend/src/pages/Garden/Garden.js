// components
import PlantCard from "../../components/PlantCard/PlantCard";

import "./Garden.css";

import { useLocation } from 'react-router-dom';


const Garden = () => {
  let { state } = useLocation();

  console.log(JSON.stringify(state));

  return (
    <>
      <section className="card-container">
        {state.selectedPlants.map((plant, index) => (
          plant.image ? <PlantCard plant={plant} key={index} showAddButton={false}/> 
            : null
        ))}
      </section>
    </>
  );
};

export default Garden;
