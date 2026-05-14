import { memo } from "react";
import CardHeader from "@mui/material/CardHeader";
import CardMedia from "@mui/material/CardMedia";
import IconButton from "@mui/material/IconButton";
import Add from "@mui/icons-material/Add";
import Remove from "@mui/icons-material/Remove";
import AttributionPopover from "./AttributionPopover";

const PlantCardImage = memo(function PlantCardImage({
  plant,
  selected,
  showAddButton,
  togglePlant,
}) {
  return (
    <div className="plant-image-container">
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

      <div className="plant-name">
        <CardHeader
          title={plant.common}
          subheader={plant.scientific}
          subheaderTypographyProps={{
            sx: {
              paddingLeft: "4px",
            },
          }}
        />
      </div>
      <CardMedia
        component="img"
        height="350"
        image={plant.image ? plant.image.cardUrl : null}
        alt={plant.image ? plant.common : null}
      />
      {plant.image ? (
        <figcaption>
          <AttributionPopover
            caption={`© Photo by ${plant.image.author}`}
            title={plant.image.title}
            author={plant.image.author}
            license={plant.image.license}
            licenseUrl={plant.image.licenseUrl}
            originalUrl={plant.image.originalUrl}
          />
        </figcaption>
      ) : null}
    </div>
  );
});

export default PlantCardImage;
