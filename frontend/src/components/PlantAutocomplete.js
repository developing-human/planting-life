import { useEffect, useState, Fragment } from "react";
import TextField from "@mui/material/TextField";
import Autocomplete from "@mui/material/Autocomplete";
import CircularProgress from "@mui/material/CircularProgress";
import openPlantsStream, { fetchPlantsByName } from "../utilities/plant-api";

export default function PlantAutocomplete({
  setPlants,
  selectedPlants,
  setDiscoverPlants,
}) {
  const [open, setOpen] = useState(false);
  const [options, setOptions] = useState([]);
  const showLoadingText = open && options.length === 0;
  const [loading, setLoading] = useState(false);
  const [userInput, setUserInput] = useState("");
  const [timerId, setTimerId] = useState(null);

  let loadingText = "No plants found";
  if (open && userInput.length < 3) {
    loadingText = "Type at least three letters to search";
  } else if (open && loading) {
    loadingText = "Searching...";
  }

  const onTextChange = (e) => {
    const userInput = e.currentTarget.value;
    setUserInput(userInput);

    if (userInput.length >= 3) {
      setLoading(true);

      // A timer is set so that the backend request is only sent if the user
      // has not typed for a brief period of time.  This kind of prevents
      // spamming the backend w/ requests while the user is still typing.
      if (timerId) {
        clearTimeout(timerId);
      }

      const newTimerId = setTimeout(() => {
        fetchPlantsByName(userInput).then((plants) => {
          setOptions(plants);
          setLoading(false);
        });
      }, 500);

      setTimerId(newTimerId);
    } else {
      setOptions([]);
    }
  };

  const onSelectionChange = (event, option, reason) => {
    console.log(reason);
    if (reason !== "selectOption") {
      return;
    }

    // When adding a plant, it may be in the "discover" tab, so select it.
    setDiscoverPlants((prevPlants) => {
      const index = prevPlants.findIndex(
        (p) => p.scientific === option.scientific
      );

      const newPlants = prevPlants.slice();
      if (index >= 0) {
        newPlants[index] = { ...prevPlants[index], selected: true };
      }

      return newPlants;
    });

    openPlantsStream({ scientificName: option.scientific }, setPlants);
  };

  // Clears options when the autocomplete is closed
  // Without this, the previous options repopulate when its reopened
  useEffect(() => {
    if (!open) {
      setOptions([]);
    }
  }, [open]);

  return (
    <Autocomplete
      id="plant-search"
      sx={{ width: 300 }}
      open={open}
      onOpen={() => {
        setOpen(true);
      }}
      onClose={() => {
        setOpen(false);
      }}
      isOptionEqualToValue={(option, value) =>
        option.scientific === value.scientific || option.common === value.common
      }
      options={options}
      loading={showLoadingText}
      loadingText={loadingText}
      // Only give options which aren't already in selectPlants.
      filterOptions={(options) =>
        options.filter(
          (candidate) =>
            !selectedPlants.some(
              (existing) => existing.scientific === candidate.scientific
            )
        )
      }
      // Use a blank option label so the input clears when its selected
      getOptionLabel={() => ""}
      renderOption={(props, option) => (
        <li {...props} key={option.id} style={{ display: "inherit" }}>
          <span>{option.common}</span>
          <br />
          <span style={{ color: "gray" }}>{option.scientific}</span>
        </li>
      )}
      onChange={onSelectionChange}
      renderInput={(params) => (
        <TextField
          {...params}
          value={userInput}
          label="Add plant by name..."
          onChange={onTextChange}
          InputProps={{
            ...params.InputProps,
            endAdornment: (
              <Fragment>
                {loading ? (
                  <CircularProgress color="inherit" size={20} />
                ) : null}
                {params.InputProps.endAdornment}
              </Fragment>
            ),
          }}
        />
      )}
    />
  );
}
