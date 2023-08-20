// components
import PlantCard from "../components/PlantCard";

const GardenTab = ({ selectedPlants }) => {
  return (
    <section className="card-container">
      {selectedPlants.map((plant) => (
        <PlantCard plant={plant} key={plant.id} showAddButton={false} />
      ))}
    </section>
  );
};

export default GardenTab;
