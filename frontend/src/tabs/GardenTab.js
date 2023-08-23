import PlantCard from "../components/PlantCard";
import GardenSummary from "../components/GardenSummary";
import "./GardenTab.css";

const GardenTab = ({ garden }) => {
  return garden ? (
    <>
      <GardenSummary garden={garden} />

      <section className="card-container">
        {garden.plants.map((plant) => (
          <PlantCard plant={plant} key={plant.id} showAddButton={false} />
        ))}
      </section>
    </>
  ) : null;
};

export default GardenTab;
