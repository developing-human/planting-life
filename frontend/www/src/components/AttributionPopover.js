import { useState } from "react";

import Popover from "@mui/material/Popover";
import Typography from "@mui/material/Typography";
import Link from "@mui/material/Link";

import "./AttributionPopover.css";

function AttributionPopover({
  caption,
  title,
  author,
  license,
  licenseUrl,
  originalUrl,
}) {
  let captionUppercase = caption.toUpperCase();
  let captionLink =
    captionUppercase.length > 40
      ? captionUppercase.substring(0, 40) + "..."
      : captionUppercase;
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
    <>
      <Link
        underline="none"
        aria-describedby={id}
        onClick={handleClick}
        sx={{ fontSize: "10px", paddingBottom: "4px", cursor: "pointer" }}
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
            <Link href={originalUrl} target="_blank">
              "{title}"
            </Link>
            <br />
            <span className="author">by {author}</span>
            <br />
            <Link
              className="license-link"
              variant="body2"
              href={licenseUrl}
              target="_blank"
              sx={{ fontSize: "12px" }}
            >
              {license}
            </Link>
          </div>
        </Typography>
      </Popover>
    </>
  );
}

export default AttributionPopover;
