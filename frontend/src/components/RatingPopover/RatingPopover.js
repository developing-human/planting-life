import { useState } from "react";

import Popover from "@mui/material/Popover";
import Typography from "@mui/material/Typography";
import Link from "@mui/material/Link";
import InfoOutlined from "@mui/icons-material/InfoOutlined";

import "./RatingPopover.css";

function RatingPopover({ id, header, text }) {
  const [anchorEl, setAnchorEl] = useState(null);

  const handleClick = (event) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const open = Boolean(anchorEl);
  const openId = open ? `${id}-popover` : undefined;

  return (
    <>
      <Link
        underline="none"
        aria-describedby={openId}
        onClick={handleClick}
        sx={{ cursor: "pointer" }}
      >
        <InfoOutlined className="ratings-info-icon" />
      </Link>
      <Popover
        id={openId}
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "center",
        }}
      >
        <div className="ratings-popover-content">
          <Typography sx={{ p: 1.5 }}>
            <h4>{header}</h4>
            {text}
          </Typography>
        </div>
      </Popover>
    </>
  );
}

export default RatingPopover;
