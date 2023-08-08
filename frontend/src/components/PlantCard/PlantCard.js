// attribution popover component
import AttributionPopover from "../AttributionPopover/AttributionPopover";
import Highlight from "../Highlight/Highlight";

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
    <Card className="plant-card"
          raised={true}
          sx={{ width: 350, maxWidth: "90vw", minHeight: 530, maxHeight: 530 }}>

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

      <CardHeader title={plant.common} subheader={plant.scientific} />
      <Grid container spacing={0}>
        <Grid item xs={6.25}>
          <CardContent>
            <Typography variant="body2" color="text.secondary">
              <div className="highlight-container">
              {plant.highlights.map((highlight) => (
                <>
                  <Highlight label={highlight.label} category={highlight.category}/>
                  <br />
                </>
              ))}
              </div>
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
        <Grid item xs={5.75}>
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
