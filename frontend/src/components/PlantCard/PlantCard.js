// attribution popover component
import AttributionPopover from "../AttributionPopover/AttributionPopover";
import RatingPopover from "../RatingPopover/RatingPopover";

// material ui
import Card from "@mui/material/Card";
import CardHeader from "@mui/material/CardHeader";
import CardMedia from "@mui/material/CardMedia";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Grid from "@mui/material/Grid";
import CircularProgress from "@mui/material/CircularProgress"

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
                  <span>
                    Pollinators: {plant.pollinatorRating.rating} / 10
                    <RatingPopover id="pollinator-rating" 
                                   header="Explanation of Pollinator Rating" 
                                   text={plant.pollinatorRating.reason} />
                  </span>
                  : null
              }
              <br/> 

              {plant.birdRating ? 
                  <span>
                    Birds: {plant.birdRating.rating} / 10
                    <RatingPopover id="bird-rating" 
                                   header="Explanation of Bird Rating" 
                                   text={plant.birdRating.reason} />
                  </span>
                  : null
              }
              <br/> 

              {plant.animalRating ? 
                  <span>
                    Animals: {plant.animalRating.rating} / 10
                    <RatingPopover id="animal-rating" 
                                   header="Explanation of Animal Rating" 
                                   text={plant.animalRating.reason} />
                  </span>
                  : null
              }
              <br /><br />
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
              {plant.bloom ? <span>Bloom: {plant.bloom}</span> : null}<br/>
              {plant.height ? <span>Height: {plant.height}</span> : null}<br/>
              {plant.spread ? <span>Spread: {plant.spread}</span> : null}<br/>
              <br />

              {plant.doneLoading ? 
                null : 
                <div className="card-loading">
                  <CircularProgress size={20} color="success"/>
                </div>
              }
            </Typography>
          </CardContent>
        </Grid>
      </Grid>
    </Card>
  );
}

export default PlantCard;
