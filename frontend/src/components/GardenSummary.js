import { useState } from "react";
// components
import Box from "@mui/material/Box";
import TextField from "@mui/material/TextField";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Tooltip from "@mui/material/Tooltip";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import useMediaQuery from "@mui/material/useMediaQuery";

import { CopyToClipboard } from "react-copy-to-clipboard";

function GardenSummary({ garden }) {
  const gardenUrl = garden.read_id
    ? process.env.REACT_APP_GARDEN_URL_PREFIX + garden.read_id
    : null;

  const [copied, setCopied] = useState(false);
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
              value={"https://planting.life/g/bfl2q"}
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
        </CardContent>
      </Card>
    </Box>
  ) : null;
}

export default GardenSummary;
