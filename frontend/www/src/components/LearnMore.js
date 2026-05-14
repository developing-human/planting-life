import { memo } from "react";
import Typography from "@mui/material/Typography";

const LearnMore = memo(function LearnMore({ plant }) {
  return (
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
  );
});

export default LearnMore;
