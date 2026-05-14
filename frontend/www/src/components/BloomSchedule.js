import Grid from "@mui/material/Grid";
import Chip from "@mui/material/Chip";

const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

const BloomSchedule = ({ bloomMonths = [] }) => {
  return (
    <Grid container>
      <Grid
        item
        xs={12}
        sx={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexDirection: "column",
        }}
      ></Grid>
      {MONTHS.map((month) => (
        <Grid item xs={3} key={month}>
          <Chip
            label={month}
            variant="outlined"
            sx={{
              backgroundColor: bloomMonths.includes(month)
                ? "lightgreen"
                : "transparent",
            }}
          />
        </Grid>
      ))}
    </Grid>
  );
};

export default BloomSchedule;
