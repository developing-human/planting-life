// components
import PlantCard from "../../components/PlantCard/PlantCard";

import "./Garden.css";

import { useLocation, useParams } from 'react-router-dom';


const Garden = () => {
  let { state } = useLocation();
  let { id } = useParams();

  console.log("in Garden");
  if (!state) {
    //TODO: Request data from GET /gardens/:id about the list of plants.  
    //      Should return OBJECT with:
    //      list of plants
    //      future: list of nurseries
    //      future: initial search location
    //      future: read or write ui
    //      future: name?
    //      future: description?
    state = {plants: []};
  }

  //TODO: Save button will POST to /gardens and receive back a url or id.
  //      Make it an object, because later we'll get back read & write urls.

  return (
    <>
      <span>{id}</span>
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
