import PlantCard from "../components/PlantCard";
import GardenSummary from "../components/GardenSummary";
import "./GardenTab.css";

const GardenTab = ({ garden, onNewGarden, setGarden, setPlants }) => {
  return garden ? (
    <>
      <GardenSummary
        garden={garden}
        onNew={onNewGarden}
        setGarden={setGarden}
      />

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
