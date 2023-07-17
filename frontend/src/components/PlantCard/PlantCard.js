// attribution popover component
import AttributionPopover from "../AttributionPopover/AttributionPopover";

// material ui
import Card from "@mui/material/Card";
import CardHeader from "@mui/material/CardHeader";
import CardMedia from "@mui/material/CardMedia";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";

// styling
import "./PlantCard.css"

function PlantCard({ plant }) {
  return (
    <Card sx={{ width: 350, maxWidth: "90vw", minHeight: 575, maxHeight: 575 }}>
      <CardHeader title={plant.common} subheader={plant.scientific} />

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
            link={plant.image.licenseUrl}
          />
        </figcaption>
      ) : null}

      <CardContent>
        <Typography variant="body2" color="text.secondary">
          {
            //plant.bloom ? "Blooms in " + plant.bloom.toLowerCase() + ". " : null
          }
    {
      //{plant.description}
    }
          {plant.pollinatorRating ? 
              <span title={plant.pollinatorRating.reason}>
                Pollinators: {plant.pollinatorRating.rating} / 10<br/> 
              </span>
              : null
          }

          {plant.birdRating ? 
              <span title={plant.birdRating.reason}>
                Bird: {plant.birdRating.rating} / 10<br/> 
              </span>
              : null
          }

          {plant.animalRating ? 
              <span title={plant.animalRating.reason}>
                Animal: {plant.animalRating.rating} / 10<br/> 
              </span>
              : null
          }
          
        </Typography>
      </CardContent>
    </Card>
  );
}

export default PlantCard;
