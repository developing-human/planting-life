import { useState } from "react";

// material ui
import Box from "@mui/material/Box";
import InputLabel from "@mui/material/InputLabel";
import MenuItem from "@mui/material/MenuItem";
import FormControl from "@mui/material/FormControl";
import Select from "@mui/material/Select";

function DropdownSelect({ id, label, options, onChange, value }) {
  // use state to handle selected option
  const [option, setOption] = useState(value);

  const handleChange = (event) => {
    setOption(event.target.value);

    if (onChange) {
      onChange(event.target.value);
    }
  };

  return (
    <Box sx={{ minWidth: 120 }}>
      <FormControl fullWidth>
        <InputLabel id={`${id}-input-label`} htmlFor={`${id}-select`}>
          {label}
        </InputLabel>

        <Select
          labelId={`${id}-input-label`}
          id={`${id}-select`}
          value={option}
          label={`${label}`}
          onChange={handleChange}
        >
          {Array.isArray(options)
            ? options.map((option) => (
                <MenuItem key={option.toLowerCase()} value={option}>
                  {option}
                </MenuItem>
              ))
            : Object.entries(options).map(([key, value]) => (
                <MenuItem key={value.toLowerCase()} value={value}>
                  {key}
                </MenuItem>
              ))}
        </Select>
      </FormControl>
    </Box>
  );
}

export default DropdownSelect;
