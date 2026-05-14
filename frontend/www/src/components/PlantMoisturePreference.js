import { memo } from "react";
import Grid from "@mui/material/Grid";
import Box from "@mui/material/Box";
import OpacityIcon from "@mui/icons-material/Opacity";
import WaterDrop from "@mui/icons-material/WaterDrop";
import WaterDropOutlined from "@mui/icons-material/WaterDropOutlined";

const PlantMoisturePreference = memo(function PlantMoisturePreference({
  moisture,
}) {
  const getMoistureColor = (level) => {
    if (!moisture) return "lightgrey";
    switch (level) {
      case "Low":
        return moisture.includes("Low") ? "royalblue" : "lightgrey";
      case "Medium":
        return moisture.includes("Medium") ? "royalblue" : "lightgrey";
      case "High":
        return moisture.includes("High") ? "royalblue" : "lightgrey";
      default:
        return "lightgrey";
    }
  };

  return (
    <Grid
      item
      xs={3.5}
      sx={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexDirection: "column",
      }}
    >
      <Box sx={{ display: "flex" }}>
        <WaterDropOutlined sx={{ color: getMoistureColor("Low") }} />
        <OpacityIcon sx={{ color: getMoistureColor("Medium") }} />
        <WaterDrop sx={{ color: getMoistureColor("High") }} />
      </Box>
    </Grid>
  );
});

export default PlantMoisturePreference;
