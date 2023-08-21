// components
import PlantCard from "../components/PlantCard";

const GardenTab = ({ garden }) => {
  return garden ? (
    <section className="card-container">
      {garden.plants.map((plant) => (
        <PlantCard plant={plant} key={plant.id} showAddButton={false} />
      ))}
    </section>
  ) : null;
};

export default GardenTab;
