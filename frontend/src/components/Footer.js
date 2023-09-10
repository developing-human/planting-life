import { Box, Link, Modal, Typography } from "@mui/material";
import { useState } from "react";

import "./Footer.css";

function Footer() {
  const [attributionsModalOpen, setAttributionsModalOpen] = useState(false);
  const handleOpen = () => setAttributionsModalOpen(true);
  const handleClose = () => setAttributionsModalOpen(false);

  const style = {
    position: "absolute",
    top: "50%",
    left: "50%",
    transform: "translate(-50%, -50%)",
    outline: 0,
    width: 400,
    bgcolor: "background.paper",
    border: "2px solid #000",
    boxShadow: 24,
    p: 4,
  };

  return (
    <>
      <div className="footer-container">
        <footer>
          <Typography>Made with ❤️ in Ohio.</Typography>
          <Link onClick={handleOpen} sx={{ cursor: "pointer" }}>
            Attributions
          </Link>
        </footer>
      </div>
      <Modal
        open={attributionsModalOpen}
        onClose={handleClose}
        aria-labelledby="modal-modal-title"
      >
        <Box sx={style}>
          <Typography id="modal-modal-title" variant="h5" component="h2">
            Attributions
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="https://simplemaps.com">
              SimpleMaps
            </Link>{" "}
            (
            <Link target="_blank" href="https://simplemaps.com/data/us-zips">
              Zipcodes
            </Link>
            ,&nbsp;
            <Link target="_blank" href="https://simplemaps.com/data/us-cities">
              Cities
            </Link>
            )
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Used to look up city names based on zipcode.
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="https://flickr.com">
              Flickr
            </Link>
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Used to find plant images. Individual photographers attributed on
            images.
          </Typography>
        </Box>
      </Modal>
    </>
  );
}

export default Footer;
