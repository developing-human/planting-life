import { Box, Link, Modal, Typography } from "@mui/material";
import { useState } from "react";
import GitHubIcon from '@mui/icons-material/GitHub';

import "./Footer.css";

function Footer() {
  const [attributionsModalOpen, setAttributionsModalOpen] = useState(false);
  const handleAttributionsOpen = () => setAttributionsModalOpen(true);
  const handleAttributionsClose = () => setAttributionsModalOpen(false);
  const [friendsModalOpen, setFriendsModalOpen] = useState(false);
  const handleFriendsOpen = () => setFriendsModalOpen(true);
  const handleFriendsClose = () => setFriendsModalOpen(false);

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
          <Link onClick={handleAttributionsOpen} sx={{ cursor: "pointer" }}>
            Attributions
          </Link>
          <span> | </span>
          <Link onClick={handleFriendsOpen} sx={{ cursor: "pointer" }}>
            Friends of Planting Life
          </Link>
          <span> | </span>
          <Link href="https://github.com/developing-human/planting-life-data" target="_blank" rel="noreferrer">
            <GitHubIcon fontSize="small" /> Contribute
          </Link>
        </footer>
      </div>
      <Modal
        open={attributionsModalOpen}
        onClose={handleAttributionsClose}
        aria-labelledby="modal-modal-title"
      >
        <Box sx={style}>
          <Typography id="modal-modal-title" variant="h5" component="h2">
            Attributions
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="https://plants.usda.gov">
              USDA
            </Link>
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Used to verify native status of plants based on location.
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="https://www.wildflower.org/plants/">
              Wildflower (University of Texas)
            </Link>
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Used to look up growing conditions.
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
      <Modal
        open={friendsModalOpen}
        onClose={handleFriendsClose}
        aria-labelledby="modal-modal-title"
      >
        <Box sx={style}>
          <Typography id="modal-modal-title" variant="h5" component="h2">
            Friends of Planting Life
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="http://sustainablewesterville.org">
              Sustainable Westerville
            </Link>
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Encourages environmental, social, and economic sustainability in Westerville, OH.
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="http://www.experienceworthington.com/greenteam">
              Worthington Green Team
            </Link>
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Encourages environmental sustainability in Worthington, OH.
          </Typography>
          <Typography variant="h6" sx={{ mt: 2 }}>
            <Link target="_blank" href="https://naturedads.com/">
              Nature Dads
            </Link>
          </Typography>
          <Typography variant="body" sx={{ mt: 2 }}>
            Encourages parents to raise environmentally conscious children.
          </Typography>
        </Box>
      </Modal>
    </>
  );
}

export default Footer;
