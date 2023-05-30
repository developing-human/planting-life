// material ui
import Card from "@mui/material/Card";
import CardHeader from "@mui/material/CardHeader";
import CardMedia from "@mui/material/CardMedia";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";

// attribution popover component
import AttributionPopover from "./AttributionPopover";

function PlantCard({ plant }) {
  return (
    <Card sx={{ width: 600 }}>
      <CardHeader title={plant.common} subheader={plant.scientific} />

      <CardMedia
        component="img"
        height="350"
        image={plant.card_url}
        alt={plant.card_url ? plant.common : null}
      />
      {plant.author ? (
        <figcaption>
          <AttributionPopover
            caption={`© Photo by ${plant.author}`}
            title={plant.title}
            author={plant.author}
            license={plant.license}
            link={plant.licenseUrl}
          />
        </figcaption>
      ) : null}

      <CardContent>
        <Typography variant="body2" color="text.secondary">
          {plant.bloom ? "Blooms in " + plant.bloom.toLowerCase() + ". " : null}
          {plant.description}
        </Typography>
      </CardContent>
    </Card>
  );
}

export default PlantCard;
