import { useState } from 'react';

import Popover from '@mui/material/Popover';
import Typography from '@mui/material/Typography';

function AttributionPopover({ caption, title, author, license }) {
    const [anchorEl, setAnchorEl] = useState(null);

    const handleClick = (event) => {
      setAnchorEl(event.currentTarget);
    };
  
    const handleClose = () => {
      setAnchorEl(null);
    };
  
    const open = Boolean(anchorEl);
    const id = open ? `${title}-simple-popover` : undefined;
  
    return (
      <div>
        <a aria-describedby={id} variant="contained" onClick={handleClick}>
          {caption}
        </a>
        <Popover
          id={id}
          open={open}
          anchorEl={anchorEl}
          onClose={handleClose}
          anchorOrigin={{
            vertical: 'bottom',
            horizontal: 'left',
          }}
        >
          <Typography sx={{ p: 2 }}>
            <div>
            {title}<br/>
            By {author}<br/>
            {license}
            </div>
          </Typography>
        </Popover>
      </div>
    )
}

export default AttributionPopover;