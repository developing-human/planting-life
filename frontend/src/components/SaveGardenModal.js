import { useEffect, useState } from "react";
// components
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Typography from "@mui/material/Typography";
import SaveIcon from "@mui/icons-material/Save";
import ClearIcon from "@mui/icons-material/Clear";
import Modal from "@mui/material/Modal";

import { IconButton, Input, InputAdornment } from "@mui/material";

function SaveGardenModal({ garden, setGarden, isOpen, setIsOpen, onSave }) {
  // This is separate from garden.name to allow reasonable behavior when
  // the edit is canceled.  It only comes back to the garden on save.
  const [gardenName, setGardenName] = useState("");

  // When the modal opens, set the name from the garden.
  useEffect(() => {
    if (isOpen) {
      setGardenName(garden.name);
    }
  }, [isOpen, garden.name]);

  return garden ? (
    <Modal open={isOpen} onClose={() => setIsOpen(false)}>
      <Box
        sx={{
          position: "absolute",
          top: "35%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          width: 400,
          bgcolor: "background.paper",
          boxShadow: 24,
          p: 4,
        }}
      >
        <Typography id="modal-modal-title" variant="h6" component="h2">
          Save Garden As...
        </Typography>
        <Input
          id="garden-name"
          label="Garden Name"
          value={gardenName}
          onChange={(event) => {
            setGardenName(event.target.value);
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
                  setGardenName("");
                  document.getElementById("garden-name").focus();
                }}
              >
                <ClearIcon />
              </IconButton>
            </InputAdornment>
          }
        />
        <Box
          sx={{
            marginTop: "15px",
          }}
        >
          <Button
            variant="contained"
            color="primary"
            startIcon={<SaveIcon />}
            onClick={() => {
              setGarden((prevGarden) => {
                return { ...prevGarden, name: gardenName, needsSave: true };
              });
              setIsOpen(false);
            }}
          >
            Save
          </Button>
          <Button
            variant="outlined"
            sx={{ marginLeft: "10px" }}
            onClick={() => {
              setIsOpen(false);
            }}
          >
            Cancel
          </Button>
        </Box>
      </Box>
    </Modal>
  ) : null;
}

export default SaveGardenModal;
