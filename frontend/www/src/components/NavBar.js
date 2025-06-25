import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import IconButton from "@mui/material/IconButton";

import "./NavBar.css";

function NavBar() {
  return (
    <Box className="nav-container">
      <AppBar
        position="static"
        sx={{
          backgroundColor: "#45a049",
          paddingInline: 0,
          height: "50px",
        }}
      >
        <Toolbar
          variant="dense"
          sx={{
            display: "flex",
            flexWrap: "wrap",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <a href="/">
            <div style={{ display: "flex" }}>
              <IconButton disabled size="string" edge="start" color="inherit">
                <img
                  className="icon"
                  src="https://planting.life/favicon-32x32.png"
                  alt="icon"
                />
              </IconButton>

              <Typography
                variant="h6"
                component="div"
                sx={{
                  flexGrow: 1,
                  paddingTop: "12px",
                  paddingLeft: "5px",
                }}
              >
                Planting Life
              </Typography>
            </div>
          </a>

          <Typography
            id="slogan"
            variant="subtitle1"
            component="div"
            sx={{
              flexGrow: 1,
              fontStyle: "italic",
              textAlign: "right",
              width: "40vw",
              maxWidth: "500px",
              paddingTop: "15px",
              paddingBottom: "5px",
              paddingLeft: "10px",
              fontSize: "0.9rem",
            }}
          >
            Plant native. Support wildlife. Grow your local ecosystem.
          </Typography>
        </Toolbar>
      </AppBar>
    </Box>
  );
}

export default NavBar;
