import { useState, useEffect } from "react";

// components
import ConditionsForm from "../../components/ConditionsForm/ConditionsForm";
import Spinner from "../../components/Spinner/Spinner";
import PlantCard from "../../components/PlantCard/PlantCard";

// material ui & styling
import Alert from "@mui/material/Alert";
import "./Home.css";

const Home = (plants, loading, error) => {
  return (
    <div>
      <ConditionsForm />

      //!TODO -- I want to bring this back in, but need to figure out how to get plants, loading, and error values
      {/* {error ? <Alert severity="error">{error}</Alert> : null}

      <section id="returned-plants">
        {plants.map((plant, index) => (
          <PlantCard plant={plant} key={index} />
        ))}

        {loading ? <Spinner /> : null}
      </section> */}
    </div>
  );
};

export default Home;
