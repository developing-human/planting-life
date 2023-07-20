// attribution popover component
import AttributionPopover from "../AttributionPopover/AttributionPopover";

// material ui
import Card from "@mui/material/Card";
import CardHeader from "@mui/material/CardHeader";
import CardMedia from "@mui/material/CardMedia";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Grid from "@mui/material/Grid";

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

      <Grid container spacing={0}>
        <Grid item xs={6}>
          <CardContent>
            <Typography variant="body2" color="text.secondary">
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
              <br />
              {plant.wikiSource ? 
                  <a href={plant.wikiSource} target="_blank" rel="noreferrer">Wikipedia</a> 
                  : null}
              {plant.usdaSource && plant.wikiSource ? <span> | </span> : null}
              {plant.usdaSource ? 
                  <a href={plant.usdaSource} target="_blank" rel="noreferrer">USDA</a> 
                  : null}
            </Typography>
          </CardContent>
        </Grid>
        <Grid item xs={6}>
          <CardContent>
            <Typography variant="body2" color="text.secondary">
              {plant.bloom ? <span>Bloom: {plant.bloom}<br/></span> : null}
              {plant.height ? <span>Height: {plant.height}<br/></span> : null}
              {plant.spread ? <span>Spread: {plant.spread}<br/></span> : null}
            </Typography>
          </CardContent>
        </Grid>
      </Grid>
    </Card>
  );
}

export default PlantCard;
