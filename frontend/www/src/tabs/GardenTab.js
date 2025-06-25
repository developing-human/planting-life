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

  // Having a read_id without a write id means the user got here through a read
  // only link.  Both missing just means they haven't saved yet.
  const readOnly = !garden.write_id && garden.read_id;

  return garden ? (
    <>
      <GardenSummary
        garden={garden}
        onNew={onNewGarden}
        setGarden={setGarden}
        readOnly={readOnly}
      />

      {!readOnly ? (
        <Box
          sx={{
            display: "flex",
            justifyContent: "center",
            marginBottom: "20px",
          }}
        >
          <PlantAutocomplete
            setPlants={setGardenPlants}
            selectedPlants={garden.plants}
            setDiscoverPlants={setPlants}
          />
        </Box>
      ) : null}

      <section className="card-container">
        {garden.plants.map((plant) => (
          <PlantCard
            plant={plant}
            key={plant.id}
            setGarden={setGarden}
            setPlants={setPlants}
            highlightSelected={false}
            showAddButton={!readOnly}
          />
        ))}
      </section>
    </>
  ) : null;
};

export default GardenTab;
