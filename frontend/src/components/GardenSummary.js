import { useState } from "react";
// components
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import TextField from "@mui/material/TextField";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Tooltip from "@mui/material/Tooltip";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import ReplayIcon from "@mui/icons-material/Replay";
import SaveIcon from "@mui/icons-material/Save";
import useMediaQuery from "@mui/material/useMediaQuery";

import { CopyToClipboard } from "react-copy-to-clipboard";
import SaveGardenModal from "./SaveGardenModal";

function GardenSummary({ garden, onNew, setGarden }) {
  const gardenUrl = garden.read_id
    ? process.env.REACT_APP_GARDEN_URL_PREFIX + garden.read_id
    : null;

  const [copied, setCopied] = useState(false);
  const [saveModalOpen, setSaveModalOpen] = useState(false);
  const narrowScreen = useMediaQuery("(max-width: 400px");

  return garden ? (
    <Box sx={{ display: "flex", justifyContent: "center" }}>
      <Card
        sx={{
          minWidth: 275,
          maxWidth: 600,
          marginBottom: "20px",
          textAlign: "center",
          display: "inline-flex",
          padding: narrowScreen ? "10px 10px 10px 10px" : "10px 50px 10px 50px",
        }}
      >
        <CardContent sx={{ margin: "auto" }}>
          <Typography variant="h5">{garden.name}</Typography>

          {gardenUrl ? (
            <TextField
              id="share-garden-url"
              label="Share this Garden"
              value={gardenUrl}
              sx={{
                marginTop: "25px",
                minWidth: "275px",
                "& .MuiInputBase-input.Mui-disabled": {
                  WebkitTextFillColor: "inherit",
                },
                "& .MuiFormLabel-root.Mui-disabled": {
                  color: "inherit",
                },
              }}
              inputProps={{
                sx: {
                  backgroundColor: "#e4e4e4",
                  borderRadius: "0 3px 3px 0",
                  paddingLeft: "15px",
                  marginLeft: "3px",
                },
              }}
              InputProps={{
                startAdornment: (
                  <CopyToClipboard
                    text={gardenUrl}
                    onCopy={() => setCopied(true)}
                  >
                    <Tooltip
                      title={copied ? "Copied to Clipboard" : "Copy Permalink"}
                      placement="top"
                      enterTouchDelay={0}
                      onClose={() => setCopied(false)}
                    >
                      <ContentCopyIcon
                        sx={{
                          cursor: "pointer",
                          color: "action.active",
                          mr: 1,
                          my: 1,
                        }}
                      />
                    </Tooltip>
                  </CopyToClipboard>
                ),
              }}
              variant="outlined"
              disabled={true}
            />
          ) : null}
          <Box sx={{ marginTop: "15px" }}>
            <Button
              sx={{ padding: "5px 10px 5px 10px" }}
              variant="outlined"
              startIcon={<ReplayIcon />}
              onClick={onNew}
            >
              New Garden
            </Button>
            <Button
              variant="contained"
              color="primary"
              sx={{ marginLeft: "10px", padding: "5px 10px 5px 10px" }}
              startIcon={<SaveIcon />}
              onClick={() => setSaveModalOpen(true)}
            >
              Save As...
            </Button>
            <SaveGardenModal
              isOpen={saveModalOpen}
              setIsOpen={setSaveModalOpen}
              garden={garden}
              setGarden={setGarden}
            />
          </Box>
        </CardContent>
      </Card>
    </Box>
  ) : null;
}

export default GardenSummary;
