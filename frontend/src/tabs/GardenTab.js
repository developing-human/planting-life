import PlantCard from "../components/PlantCard";
import GardenSummary from "../components/GardenSummary";
import "./GardenTab.css";
import PlantAutocomplete from "../components/PlantAutocomplete";
import { Box } from "@mui/material";

const GardenTab = ({ garden, onNewGarden, setGarden, setPlants }) => {
  const setGardenPlants = (updateFunction) => {
    setGarden((prevGarden) => ({
      ...prevGarden,
      plants: updateFunction(prevGarden.plants),
      needsSave: true,
    }));
  };

  return garden ? (
    <>
      <GardenSummary
        garden={garden}
        onNew={onNewGarden}
        setGarden={setGarden}
      />

      <Box
        sx={{ display: "flex", justifyContent: "center", marginBottom: "20px" }}
      >
        <PlantAutocomplete
          setPlants={setGardenPlants}
          selectedPlants={garden.plants}
        />
      </Box>

      <section className="card-container">
        {garden.plants.map((plant) => (
          <PlantCard
            plant={plant}
            key={plant.id}
            setGarden={setGarden}
            setPlants={setPlants}
            highlightSelected={false}
          />
        ))}
      </section>
    </>
  ) : null;
};

export default GardenTab;
