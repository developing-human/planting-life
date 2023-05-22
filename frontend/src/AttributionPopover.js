import { useState } from "react";

import Popover from "@mui/material/Popover";
import Typography from "@mui/material/Typography";
import Link from "@mui/material/Link";

import "./AttributionPopover.css";

function AttributionPopover({ caption, title, author, license, link }) {
  let captionLink = caption.toUpperCase().substring(0,23) + "...";
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
      <Link
        underline="none"
        aria-describedby={id}
        onClick={handleClick}
        sx={{ fontSize: "10px", paddingBottom: "4px" }}
      >
        {captionLink}
      </Link>
      <Popover
        id={id}
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "center",
        }}
      >
        <Typography sx={{ p: 1.5 }}>
          <div id="popover">
            "{title}"<br />
            <span className="author">by {author}</span>
            <br />
            <Link
              className="license-link"
              variant="body2"
              href={link}
              target="blank"
              sx={{ fontSize: "12px" }}
            >
              {license}
            </Link>
          </div>
        </Typography>
      </Popover>
    </div>
  );
}

export default AttributionPopover;
