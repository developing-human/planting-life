import * as React from "react";
import { styled } from "@mui/material/styles";
import ArrowForwardIosSharpIcon from "@mui/icons-material/ArrowForwardIosSharp";
import MuiAccordion from "@mui/material/Accordion";
import MuiAccordionSummary from "@mui/material/AccordionSummary";
import MuiAccordionDetails from "@mui/material/AccordionDetails";
import Typography from "@mui/material/Typography";

const Accordion = styled((props) => (
  <MuiAccordion disableGutters elevation={0} square {...props} />
))(({ theme }) => ({
  border: `1px solid ${theme.palette.divider}`,
  "&:not(:last-child)": {
    borderBottom: 0,
  },
  "&:before": {
    display: "none",
  },
}));

const AccordionSummary = styled((props) => (
  <MuiAccordionSummary
    expandIcon={<ArrowForwardIosSharpIcon sx={{ fontSize: "0.9rem" }} />}
    {...props}
  />
))(({ theme }) => ({
  backgroundColor:
    theme.palette.mode === "dark"
      ? "rgba(255, 255, 255, .05)"
      : "rgba(0, 0, 0, .03)",
  flexDirection: "row-reverse",
  "& .MuiAccordionSummary-expandIconWrapper.Mui-expanded": {
    transform: "rotate(90deg)",
  },
  "& .MuiAccordionSummary-content": {
    marginLeft: theme.spacing(1),
  },
}));

const AccordionDetails = styled(MuiAccordionDetails)(({ theme }) => ({
  padding: theme.spacing(2),
  borderTop: "1px solid rgba(0, 0, 0, .125)",
}));

function IntroAccordion({ expanded, setExpanded }) {
  const handleChange = (panel) => (event, newExpanded) => {
    // if not expanded, expand the targeted panel, else pass false to close it
    setExpanded(newExpanded ? panel : false);
  };

  return (
    <div>
      <Accordion
        expanded={expanded === "welcome"}
        onChange={handleChange("welcome")}
      >
        <AccordionSummary aria-controls="welcome-content" id="welcome-header">
          <Typography>Why plant native?</Typography>
        </AccordionSummary>
        <AccordionDetails>
          <Typography>
            Planting native celebrates the beauty of our surroundings while
            creating a haven for local wildlife, nurturing a sense of belonging,
            and reminding us that we are an integral part of the intricate
            tapestry of life on this Earth.
            <br />
            <br />
            To find native plants that would thrive in your area, simply enter
            your zip code along with the amount of shade and soil moisture level
            where you intend to plant.
          </Typography>
        </AccordionDetails>
      </Accordion>
    </div>
  );
}

export default IntroAccordion;
