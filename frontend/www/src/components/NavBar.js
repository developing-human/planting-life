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
            background: "linear-gradient(to bottom, #66bb6a, #2e7d32)",
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
              height: "100%",
            }}
          >
            <a href="/" style={{ textDecoration: "none", color: "inherit" }}>
              <div style={{ display: "flex", alignItems: "center" }}>
                <IconButton disabled size="small" edge="start" color="inherit" sx={{ p: 0, mr: 1 }}>
                  <img
                    className="icon"
                    src="https://planting.life/favicon-32x32.png"
                    alt="icon"
                    style={{ width: "32px", height: "32px" }}
                  />
                </IconButton>

                <Typography
                  variant="h6"
                  component="div"
                  sx={{
                    fontFamily: "var(--font-heading)",
                    fontWeight: "bold",
                    color: "var(--color-cream)",
                    fontSize: "1.25rem",
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
                fontFamily: "var(--font-body)",
                fontWeight: 300,
                textAlign: "right",
                width: { xs: "auto", md: "40vw" },
                maxWidth: "500px",
                fontSize: "0.85rem",
                color: "rgba(253, 252, 240, 0.8)",
                display: { xs: "none", sm: "block" },
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
