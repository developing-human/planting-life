import { useState } from 'react';
import Box from '@mui/material/Box';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import Select from '@mui/material/Select';

function DropdownSelect( {id, label, options, onChange} ) {
  // use state to handle selected option
  const [option, setOption] = useState('');

  const handleChange = (event) => {
    setOption(event.target.value);

    if (onChange) {
      onChange(event.target.value);
    }
  };

  return (
    <Box sx={{ minWidth: 120 }}>
      <FormControl fullWidth>
        <InputLabel id={`${id}-input-label`} htmlFor={`${id}-select`}>{label}</InputLabel>

        <Select
          labelId={`${id}-input-label`}
          id={`${id}-select`}
          value={option}
          label={`${label}`}
          onChange={handleChange}
        >

        {options.map(option => {
            return <MenuItem key={option.toLowerCase()} value={option}>{option}</MenuItem>
        })}

        </Select>
      </FormControl>
    </Box>
  );
}

export default DropdownSelect;
