import Highlight from "./Highlight";
import Typography from "@mui/material/Typography";

function Highlights({ plant }) {
  return (
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
  );
}

export default Highlights;
