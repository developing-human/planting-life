import { useEffect, useState } from "react";
// components
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import TextField from "@mui/material/TextField";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Tooltip from "@mui/material/Tooltip";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import ClearIcon from "@mui/icons-material/Clear";
import EditIcon from "@mui/icons-material/Edit";
import useMediaQuery from "@mui/material/useMediaQuery";

import { CopyToClipboard } from "react-copy-to-clipboard";
import { IconButton, Input, InputAdornment } from "@mui/material";

function GardenSummary({ garden, onNew, setGarden, readOnly }) {
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
          padding: narrowScreen ? "10px 10px 0px 10px" : "10px 50px 0px 50px",
        }}
      >
        <CardContent sx={{ margin: "auto" }}>
          {readOnly ? (
            <Typography variant="h5">
              {garden.name || "My Native Garden"}
            </Typography>
          ) : (
            <EditableGardenName
              gardenName={garden.name}
              setGarden={setGarden}
            />
          )}

          {gardenUrl && !readOnly ? (
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
          {garden.plants.length > 0 ? (
            <Box sx={{ marginTop: "15px" }}>
              <Button
                sx={{ margin: "5px", padding: "5px 10px 5px 10px" }}
                variant="outlined"
                onClick={onNew}
              >
                Start New Garden
              </Button>
              {readOnly ? (
                <Button
                  variant="outlined"
                  color="primary"
                  sx={{ margin: "5px", padding: "5px 10px 5px 10px" }}
                  onClick={() => {
                    setGarden((prevGarden) => {
                      return {
                        ...prevGarden,
                        read_id: null,
                        write_id: null,
                        needsSave: true,
                      };
                    });
                  }}
                >
                  Copy This Garden
                </Button>
              ) : null}
            </Box>
          ) : null}
        </CardContent>
      </Card>
    </Box>
  ) : null;
}

// I pulled this out for readability above, but I don't think its worth moving
// into another file since its so specific to the garden summary.  May change
// my mind on that.
function EditableGardenName({ gardenName, setGarden }) {
  const [editingName, setEditingName] = useState(false);
  const [transientGardenName, setTransientGardenName] = useState(gardenName);

  // When editingName switches to true, focus & select all text in the field.
  useEffect(() => {
    if (editingName) {
      const gardenNameElement = document.getElementById("garden-name");
      gardenNameElement.focus();
      gardenNameElement.select();
    }
  }, [editingName]);

  const handleSubmit = (event) => {
    event.preventDefault();
    setGarden((prevGarden) => {
      return {
        ...prevGarden,
        name: transientGardenName,
        needsSave: true,
      };
    });
    setEditingName(false);
  };

  return (
    <form onSubmit={handleSubmit}>
      <Typography variant="h5">
        {!editingName ? (
          gardenName || "My Native Garden"
        ) : (
          <Input
            id="garden-name"
            label="Garden Name"
            value={transientGardenName}
            onChange={(event) => {
              setTransientGardenName(event.target.value);
            }}
            variant="standard"
            required
            inputProps={{ maxLength: "255" }}
            sx={{ width: "100%" }}
            endAdornment={
              <InputAdornment position="end">
                <IconButton
                  edge="end"
                  sx={{ paddingBottom: "15px" }}
                  onClick={() => {
                    setTransientGardenName("");
                    document.getElementById("garden-name").focus();
                  }}
                >
                  <ClearIcon />
                </IconButton>
              </InputAdornment>
            }
          />
        )}
      </Typography>
      <Box sx={{ paddingTop: "5px" }}>
        {!editingName ? (
          <Button
            variant="outlined"
            sx={{ padding: "2px 7px" }}
            startIcon={<EditIcon />}
            onClick={() => {
              setTransientGardenName(gardenName);
              setEditingName(true);
            }}
          >
            Rename
          </Button>
        ) : null}
        {editingName ? (
          <>
            <Button
              variant="contained"
              type="submit"
              sx={{
                padding: "2px 7px",
                marginRight: "5px",
              }}
            >
              Save
            </Button>
            <Button
              variant="outlined"
              sx={{ padding: "2px 7px" }}
              onClick={() => setEditingName(false)}
            >
              Cancel
            </Button>
          </>
        ) : null}
      </Box>
    </form>
  );
}

export default GardenSummary;
