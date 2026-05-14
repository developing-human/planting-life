import { memo } from "react";
import Grid from "@mui/material/Grid";
import Box from "@mui/material/Box";
import Brightness5Icon from "@mui/icons-material/Brightness5";
import Brightness6Icon from "@mui/icons-material/Brightness6";
import Brightness7Icon from "@mui/icons-material/Brightness7";

const PlantShadePreference = memo(function PlantShadePreference({ shade }) {
  const getShadeColor = (level) => {
    if (!shade) return "lightgrey";
    switch (level) {
      case "Full Shade":
        return shade.includes("Full Shade") ? "orange" : "lightgrey";
      case "Partial Shade":
        return shade.includes("Partial Shade") ? "orange" : "lightgrey";
      case "Full Sun":
        return shade.includes("Full Sun") ? "orange" : "lightgrey";
      default:
        return "lightgrey";
    }
  };

  return (
    <Box sx={{ display: "flex" }}>
      <Brightness5Icon sx={{ color: getShadeColor("Full Shade") }} />
      <Brightness6Icon sx={{ color: getShadeColor("Partial Shade") }} />
      <Brightness7Icon sx={{ color: getShadeColor("Full Sun") }} />
    </Box>
  );
});

export default PlantShadePreference;
