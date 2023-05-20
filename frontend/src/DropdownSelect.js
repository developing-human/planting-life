import { useState } from 'react';
import Box from '@mui/material/Box';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import Select from '@mui/material/Select';

function DropdownSelect( {label, options} ) {
  const [option, setOption] = useState('');

  const handleChange = (event) => {
    setOption(event.target.value);
  };

  console.log(option);

  return (
    <Box sx={{ minWidth: 120 }}>
      <FormControl fullWidth>
        <InputLabel id="demo-simple-select-label">{label}</InputLabel>

        <Select
          labelId="demo-simple-select-label"
          id="demo-simple-select"
          value={option}
          label={`${label}`}
          onChange={handleChange}
        >

        {options.map(option => {
            return <MenuItem key={option} value={option}>{option}</MenuItem>
        })}

        </Select>
      </FormControl>
    </Box>
  );
}

export default DropdownSelect;