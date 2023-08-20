import * as React from "react";
import { styled } from "@mui/material/styles";
import ArrowForwardIosSharpIcon from "@mui/icons-material/ArrowForwardIosSharp";
import MuiAccordion from "@mui/material/Accordion";
import MuiAccordionSummary from "@mui/material/AccordionSummary";
import MuiAccordionDetails from "@mui/material/AccordionDetails";
import Typography from "@mui/material/Typography";

const Accordion = styled((props) => (
  <MuiAccordion
    disableGutters
    elevation={0}
    square
    {...props}
    TransitionProps={{ timeout: 0 }}
  />
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
  textAlign: "justify",
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
          <Typography>What is Planting Life?</Typography>
        </AccordionSummary>
        <AccordionDetails>
          <Typography>
            Planting Life helps you discover native plants that will thrive in
            your garden. Whether you're a new or experienced gardener, it will
            help you choose plants which support your local ecosystem.
            <br />
            <br />
            By planting native, you'll provide food and shelter for wildlife
            which already lives near you. You'll also simplify maintenance for
            your garden by selecting plants which are well-suited to the
            moisture and soil conditions in your area.
          </Typography>
        </AccordionDetails>
      </Accordion>
    </div>
  );
}

export default IntroAccordion;
