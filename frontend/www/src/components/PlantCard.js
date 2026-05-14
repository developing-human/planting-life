import { useState, memo } from "react";

// attribution popover component
import BloomSchedule from "./BloomSchedule";
import Highlights from "./Highlights";
import PlantCardImage from "./PlantCardImage";
import PlantShadePreference from "./PlantShadePreference";
import PlantMoisturePreference from "./PlantMoisturePreference";

// material ui
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
import Grid from "@mui/material/Grid";

// styling
import "./PlantCard.css";
import { Divider } from "@mui/material";

const PlantCard = memo(function PlantCard({
  plant,
  setGarden,
  showAddButton,
  setPlants,
  highlightSelected,
}) {
  const [selected, setSelected] = useState(plant.selected || false);
  const togglePlant = () => {
    const newSelected = !selected;
    // Set the state on the PlantCard, used for rendering
    setSelected(newSelected);

    // Add or remove from the list of selected plants
    setGarden((prevGarden) => {
      let newSelectedPlants;
      if (newSelected) {
        newSelectedPlants = prevGarden.plants.concat({
          ...plant,
          selected: true,
        });
      } else {
        newSelectedPlants = prevGarden.plants.filter(
          (existing) => existing.scientific !== plant.scientific,
        );
      }

      return { ...prevGarden, plants: newSelectedPlants, needsSave: true };
    });

    // Update plants state with the flag, this will be remembered when navigating
    // back to Home from the Garden page.
    setPlants((prevPlants) => {
      const index = prevPlants.findIndex(
        (p) => p.scientific === plant.scientific,
      );
      if (index === -1) {
        return prevPlants;
      }

      const newPlants = prevPlants.slice();
      newPlants[index] = { ...prevPlants[index], selected: newSelected };
      return newPlants;
    });
  };

  return (
    <Card
      className={
        "plant-card" +
        // only add the selected class (to highlight this card) if it is
        // selected and it should be highlighted
        (highlightSelected !== false && selected ? " selected" : "")
      }
      raised={true}
      sx={{
        width: 350,
        maxWidth: "90vw",
        minHeight: 523,
        maxHeight: 523,
        borderRadius: "12px",
        position: "relative",
      }}
    >
      <PlantCardImage
        plant={plant}
        selected={selected}
        showAddButton={showAddButton}
        togglePlant={togglePlant}
      />

      <CardContent sx={{ position: "relative" }}>
        <Grid container spacing={2}>
          <Grid item xs={3.5} className="flex-center-column">
            <PlantShadePreference shade={["Full Sun"]} />
          </Grid>
          <Grid item xs={3.5} className="flex-center-column">
            <PlantMoisturePreference moisture={["Medium", "High"]} />
          </Grid>
          <Grid item xs={5}>
            <Typography variant="body2" color="text.secondary">
              {plant.height ? <span>Height: {plant.height}</span> : null}
              <br />
              {plant.spread ? <span>Spread: {plant.spread}</span> : null}
            </Typography>
          </Grid>
        </Grid>

        <Divider sx={{ paddingTop: "5px", paddingBottom: "3px" }}>
          <Typography variant="body2" color="text.secondary">
            Highlights &amp; Bloom Schedule
          </Typography>
        </Divider>

        <Grid container>
          <Grid item xs={6.5}>
            <Highlights plant={plant} />
          </Grid>
          <Grid item xs={5.5}>
            <BloomSchedule bloomMonths={["Apr", "May", "Jun"]} />
          </Grid>
        </Grid>
      </CardContent>
      <Box
        className="flex-center-column"
        sx={{
          position: "absolute",
          bottom: "5px",
          width: "100%",
        }}
      >
        <Typography
          variant="body2"
          color="text.secondary"
          sx={{ justifyContent: "center" }}
        >
          {plant.wildflowerSource ? (
            <a href={plant.wildflowerSource} target="_blank" rel="noreferrer">
              Wildflower.org
            </a>
          ) : null}
          {plant.wildflowerSource && (plant.wikiSource || plant.usdaSource) ? (
            <span> | </span>
          ) : null}
          {plant.usdaSource ? (
            <a href={plant.usdaSource} target="_blank" rel="noreferrer">
              USDA
            </a>
          ) : null}
          {plant.usdaSource && plant.wikiSource ? <span> | </span> : null}
          {plant.wikiSource ? (
            <a href={plant.wikiSource} target="_blank" rel="noreferrer">
              Wikipedia
            </a>
          ) : null}
        </Typography>
      </Box>
    </Card>
  );
});

export default PlantCard;
