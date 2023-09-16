import { useState, memo } from "react";

// attribution popover component
import AttributionPopover from "./AttributionPopover";
import Highlight from "./Highlight";

// material ui
import Card from "@mui/material/Card";
import CardHeader from "@mui/material/CardHeader";
import CardMedia from "@mui/material/CardMedia";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Grid from "@mui/material/Grid";
import CircularProgress from "@mui/material/CircularProgress";
import IconButton from "@mui/material/IconButton";
import Add from "@mui/icons-material/Add";
import Remove from "@mui/icons-material/Remove";

import Carousel from "react-material-ui-carousel";
import { LazyLoadComponent } from "react-lazy-load-image-component";
import "react-lazy-load-image-component/src/effects/blur.css";

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
          (existing) => existing.scientific !== plant.scientific
        );
      }

      return { ...prevGarden, plants: newSelectedPlants, needsSave: true };
    });

    // Update plants state with the flag, this will be remembered when navigating
    // back to Home from the Garden page.
    setPlants((prevPlants) => {
      const index = prevPlants.findIndex(
        (p) => p.scientific === plant.scientific
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
      }}
    >
      <Carousel
        autoPlay={false}
        animation="fade"
        indicators={false}
        navButtonsAlwaysVisible={true}
        swipe={false}
      >
        {plant.images.map((image, i) => (
          <div className="plant-image-container" key={i}>
            {showAddButton !== false &&
              (selected ? (
                <IconButton
                  size="large"
                  className="add-plant-button"
                  onClick={togglePlant}
                >
                  <Remove />
                </IconButton>
              ) : (
                <IconButton
                  size="large"
                  className="add-plant-button"
                  onClick={togglePlant}
                >
                  <Add />
                </IconButton>
              ))}

            {/* <LazyLoadComponent> */}
            <CardMedia
              component="img"
              height="350"
              image={image ? image.cardUrl : null}
              alt={image ? plant.common : null}
            />
            {/* </LazyLoadComponent> */}
            {image ? (
              <figcaption>
                <AttributionPopover
                  caption={`© Photo by ${image.author}`}
                  title={image.title}
                  author={image.author}
                  license={image.license}
                  licenseUrl={image.licenseUrl}
                  originalUrl={image.originalUrl}
                />
              </figcaption>
            ) : null}
          </div>
        ))}
      </Carousel>

      <CardHeader
        title={plant.common}
        subheader={plant.scientific}
        subheaderTypographyProps={{
          sx: {
            paddingLeft: "4px",
          },
        }}
      />
      <Divider variant="middle" />

      <CardContent>
        <Grid container spacing={2}>
          <Grid item xs={6.25}>
            <div className="highlight-container">
              <Typography variant="body2" color="text.secondary">
                {plant.highlights
                  ? plant.highlights.map((highlight) => (
                      <span key={plant.id + "-" + highlight.label}>
                        <Highlight
                          label={highlight.label}
                          category={highlight.category}
                        />
                        <br />
                      </span>
                    ))
                  : null}
              </Typography>
            </div>
            <Typography
              variant="body2"
              color="text.secondary"
              sx={{ marginTop: "-7px" }}
            >
              {plant.wikiSource ? (
                <a href={plant.wikiSource} target="_blank" rel="noreferrer">
                  Wikipedia
                </a>
              ) : null}
              {plant.usdaSource && plant.wikiSource ? <span> | </span> : null}
              {plant.usdaSource ? (
                <a href={plant.usdaSource} target="_blank" rel="noreferrer">
                  USDA
                </a>
              ) : null}
            </Typography>
          </Grid>
          <Grid item xs={5.75}>
            <Typography variant="body2" color="text.secondary">
              {plant.bloom ? <span>Bloom: {plant.bloom}</span> : null}
              <br />
              {plant.height ? <span>Height: {plant.height}</span> : null}
              <br />
              {plant.spread ? <span>Spread: {plant.spread}</span> : null}
              <br />
              <br />

              {plant.doneLoading ? null : (
                <span className="card-loading">
                  <CircularProgress size={20} color="success" />
                </span>
              )}
            </Typography>
          </Grid>
        </Grid>
      </CardContent>
    </Card>
  );
});

export default PlantCard;
