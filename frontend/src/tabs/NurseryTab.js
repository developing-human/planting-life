import { Box, Card, CardContent, TextField, Typography } from "@mui/material";
import Nursery from "../components/Nursery";
import SearchIcon from "@mui/icons-material/Search";
import { getNurseries } from "../utilities/nursery-api";

const NurseryTab = ({ nurseries, setNurseries, zip, setZip }) => {
  const handleZipChange = (event) => {
    const updatedZip = event.target.value;
    setZip(updatedZip);

    if (updatedZip.length === 5) {
      getNurseries(updatedZip, setNurseries);
      event.target.blur();
    }
  };

  return (
    <>
      <Box sx={{ display: "flex", justifyContent: "center" }}>
        <Card
          sx={{
            minWidth: 275,
            maxWidth: 525,
            marginBottom: "20px",
            textAlign: "center",
            display: "inline-flex",
          }}
        >
          <CardContent sx={{}}>
            <Box display="flex" justifyContent="center">
              <Typography variant="h5">
                Native-Focused Nurseries near
                <TextField
                  id="nursery-zip"
                  value={zip}
                  variant="outlined"
                  onFocus={(event) => event.target.select()}
                  onChange={handleZipChange}
                  sx={{
                    marginLeft: "5px",
                    width: "110px",
                    marginBottom: "15px",
                  }}
                  InputProps={{
                    endAdornment: (
                      <SearchIcon sx={{ color: "action.active" }} />
                    ),
                  }}
                  inputProps={{
                    inputMode: "numeric",
                    pattern: "[0-9]{5}",
                    maxLength: 5,
                    title: "US Zip Code",
                    sx: {
                      padding: "5px 0px 0px 10px",
                      fontSize: "1.2rem",
                    },
                  }}
                  onKeyPress={(event) => {
                    // Only allow numbers & Enter to be typed
                    if (!/[0-9]/.test(event.key) && event.key !== "Enter") {
                      event.preventDefault();
                    }
                  }}
                />
              </Typography>
            </Box>
            <Typography
              variant="body1"
              sx={{ paddingTop: "10px", textAlign: "justify" }}
            >
              These nurseries are committed to offering native species that are
              well-suited to your local environment. By focusing on native
              plants, these nurseries support the health of ecosystems and
              promote sustainable gardening practices.
            </Typography>
          </CardContent>
        </Card>
      </Box>
      <section className="card-container">
        {nurseries.map((nursery, index) => (
          <Nursery nursery={nursery} key={index} />
        ))}
      </section>
    </>
  );
};

export default NurseryTab;
