// components
import { Paper } from "@mui/material";
import PlantCard from "../components/PlantCard";
import "./GardenTab.css";

const GardenTab = ({ garden }) => {
  const gardenUrl = garden.read_id
    ? process.env.REACT_APP_GARDEN_URL_PREFIX + garden.read_id
    : null;

  return garden ? (
    <>
      <Paper className="garden-summary">
        <h3>{garden.name}</h3>
        {gardenUrl ? (
          <>
            <div>Share this garden</div>
            <div>
              <a href={gardenUrl}>{gardenUrl.replace(/https?:\/\//, "")}</a>
            </div>
          </>
        ) : null}
      </Paper>
      <section className="card-container">
        {garden.plants.map((plant) => (
          <PlantCard plant={plant} key={plant.id} showAddButton={false} />
        ))}
      </section>
    </>
  ) : null;
};

export default GardenTab;
