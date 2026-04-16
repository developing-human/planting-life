import React from "react";
import {
  Box,
  Container,
  Typography,
  Button,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Paper,
} from "@mui/material";
import Search from "@mui/icons-material/Search";
import ShareIcon from "@mui/icons-material/Share";
import LocationOnIcon from "@mui/icons-material/LocationOn";
import "./LandingPage.css";

const LandingPage = ({ onStartGarden }) => {
  return (
    <Box>
      {/* Hero Section */}
      <Box
        id="hero-container"
        sx={{
          backgroundImage: "url('/hero-caterpillar.jpg')",
          // "url('https://images.unsplash.com/photo-1628618342670-d06cd1800343?q=80&w=2340&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D')",
          backgroundSize: "cover",
          backgroundPosition: "center",
          minHeight: "80vh",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: { xs: 0.5, md: 8 },
          position: "relative",
          width: "100%",
          zIndex: 5,
        }}
      >
        <Container maxWidth="md" sx={{ px: { xs: 2, md: 0 } }}>
          <Box
            sx={{
              backgroundColor: "rgba(255, 255, 255, 0.15)",
              backdropFilter: "blur(12px)",
              WebkitBackdropFilter: "blur(12px)",
              borderRadius: "24px",
              padding: { xs: 2, sm: 4, md: 8 },
              textAlign: "center",
              color: "white",
              border: "1px solid rgba(255, 255, 255, 0.2)",
              boxShadow: "0 8px 32px 0 rgba(0, 0, 0, 0.37)",
            }}
          >
            <Typography
              variant="h2"
              component="h1"
              sx={{
                fontWeight: "bold",
                fontSize: { xs: "1.8rem", sm: "2.2rem", md: "3.5rem" },
                lineHeight: 1.2,
                mb: 2,
                textShadow: "0 2px 4px rgba(0,0,0,0.3)",
                color: "white",
              }}
            >
              Find native plants for
              <br /> <em>your</em> garden
            </Typography>
            <Typography
              variant="h5"
              sx={{
                fontSize: { xs: "1rem", sm: "1.1rem", md: "1.3rem" },
                mb: 0,
                fontWeight: 300,
                textShadow: "0 1px 3px rgba(0,0,0,0.3)",
                color: "rgba(255, 255, 255, 0.9)",
              }}
            >
              Create a backyard that supports local wildlife and thrives in your
              environment.
            </Typography>
            <Box
              sx={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                mb: 1,
              }}
            >
              <List
                sx={{
                  mb: 1,
                  textAlign: "left",
                  display: "inline-block",
                  color: "white",
                }}
              >
                <ListItem disableGutters sx={{ py: 0.25 }}>
                  <ListItemIcon sx={{ minWidth: 40, color: "#c8e6c9" }}>
                    <Search />
                  </ListItemIcon>
                  <ListItemText
                    primary="Find plants that fit your zip code & growing conditions"
                    primaryTypographyProps={{
                      variant: "body1",
                      style: {
                        color: "white",
                        textShadow: "0 1px 2px rgba(0,0,0,0.5)",
                      },
                    }}
                  />
                </ListItem>
                <ListItem disableGutters sx={{ py: 0.25 }}>
                  <ListItemIcon sx={{ minWidth: 40, color: "#c8e6c9" }}>
                    <ShareIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary="Create a garden plan to share with friends or nurseries"
                    primaryTypographyProps={{
                      variant: "body1",
                      style: {
                        color: "white",
                        textShadow: "0 1px 2px rgba(0,0,0,0.5)",
                      },
                    }}
                  />
                </ListItem>
                <ListItem disableGutters sx={{ py: 0.25 }}>
                  <ListItemIcon sx={{ minWidth: 40, color: "#c8e6c9" }}>
                    <LocationOnIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary="Find nurseries in your area that specialize in native plants"
                    primaryTypographyProps={{
                      variant: "body1",
                      style: {
                        color: "white",
                        textShadow: "0 1px 2px rgba(0,0,0,0.5)",
                      },
                    }}
                  />
                </ListItem>
              </List>
              <Box sx={{ textAlign: "center" }}>
                <Button
                  variant="contained"
                  color="success"
                  size="large"
                  onClick={onStartGarden}
                  sx={{
                    px: 6,
                    py: 2,
                    fontSize: "1.1rem",
                    fontWeight: "bold",
                    borderRadius: "30px",
                    boxShadow: "0 4px 20px rgba(0,0,0,0.4)",
                    transition: "transform 0.2s",
                    paddingY: 1.5,
                    paddingX: 3,
                    "&:hover": {
                      transform: "scale(1.05)",
                      backgroundColor: "#388e3c",
                    },
                  }}
                >
                  Start Your Garden
                </Button>
                <Typography
                  variant="caption"
                  display="block"
                  sx={{
                    mt: 1,
                    color: "rgba(255, 255, 255, 0.8)",
                    fontSize: "0.9rem",
                    fontWeight: 500,
                    textShadow: "0 1px 2px rgba(0,0,0,0.5)",
                  }}
                >
                  (Free, no signup)
                </Typography>
              </Box>
            </Box>
          </Box>
        </Container>
      </Box>

      {/* Why Plant Native Section */}
      <Box className="bg-light">
        <Container maxWidth="md" sx={{ py: 4, textAlign: "center" }}>
          <Typography
            variant="h3"
            component="h2"
            gutterBottom
            sx={{ fontWeight: "bold", mb: 3 }}
          >
            Why Plant Native?
          </Typography>
          <Typography
            variant="h6"
            sx={{ mb: 5, color: "text.secondary", lineHeight: 1.6 }}
          >
            Planting native is about more than just aesthetics, it's about
            supporting our local ecosystem. Native plants provide the essential
            food and shelter that local butterflies, bees, and birds need to
            thrive.
          </Typography>
          <Grid container spacing={4}>
            {[
              {
                title: "Support Pollinators",
                text: "Native plants provide the exact nectar and host leaves that local butterflies and bees need to survive.",
              },
              {
                title: "Built for Your Backyard",
                text: "Because they evolved in your region, native plants are naturally adapted to your climate, which means they generally need less water and fewer fertilizers.",
              },
              {
                title: "Restore the Balance",
                text: "Even a small garden can act as a vital stepping stone for wildlife, helping to bring nature back into our neighborhoods.",
              },
            ].map((item, index) => (
              <Grid item xs={12} sm={4} key={index}>
                <Paper
                  elevation={0}
                  sx={{
                    p: 4,
                    height: "100%",
                    borderRadius: "24px",
                    border: "1px solid rgba(76, 175, 80, 0.2)",
                    backgroundColor: "#f1f8e9",
                    transition:
                      "transform 0.2s, box-shadow 0.2s, background-color 0.2s",
                    "&:hover": {
                      transform: "translateY(-4px)",
                      boxShadow: "0 12px 24px rgba(0,0,0,0.05)",
                      backgroundColor: "#e8f5e9",
                    },
                  }}
                >
                  <Typography
                    variant="h6"
                    component="h3"
                    gutterBottom
                    sx={{ fontWeight: "bold", mb: 2 }}
                  >
                    {item.title}
                  </Typography>
                  <Typography
                    variant="body1"
                    color="text.secondary"
                    sx={{ lineHeight: 1.6 }}
                  >
                    {item.text}
                  </Typography>
                </Paper>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Gardening Made Simple Section */}
      <Box>
        <Container maxWidth="md" sx={{ py: 8, textAlign: "center" }}>
          <Typography
            variant="h3"
            component="h2"
            gutterBottom
            sx={{ fontWeight: "bold", mb: 6 }}
          >
            Gardening Made Simple
          </Typography>
          <Grid container spacing={4}>
            {[
              {
                step: "1",
                title: "Describe your space",
                text: "Enter your zip code and a few details about your yard, like how much sun it gets and how wet it is.",
              },
              {
                step: "2",
                title: "Find your matches",
                text: "Get a list of plants that are a great fit for your garden.",
              },
              {
                step: "3",
                title: "Get the details",
                text: "Dive into the details, from bloom times to whether the plants are deer-resistant.",
              },
              {
                step: "4",
                title: "Bring it to life",
                text: "Save your favorites to a garden plan that you can keep or share with a local nursery.",
              },
            ].map((item, index) => (
              <Grid item xs={12} sm={6} md={3} key={index}>
                <Paper
                  elevation={0}
                  sx={{
                    p: 4,
                    textAlign: "center",
                    height: "100%",
                    borderRadius: "24px",
                    border: "1px solid rgba(76, 175, 80, 0.2)",
                    backgroundColor: "#f1f8e9",
                    transition:
                      "transform 0.2s, box-shadow 0.2s, background-color 0.2s",
                    "&:hover": {
                      transform: "translateY(-4px)",
                      boxShadow: "0 12px 24px rgba(0,0,0,0.05)",
                      backgroundColor: "#e8f5e9",
                    },
                  }}
                >
                  <Typography
                    variant="h6"
                    component="h3"
                    gutterBottom
                    sx={{ fontWeight: "bold", mb: 2 }}
                  >
                    {item.title}
                  </Typography>
                  <Typography
                    variant="body2"
                    color="text.secondary"
                    sx={{ lineHeight: 1.6 }}
                  >
                    {item.text}
                  </Typography>
                </Paper>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Mission Section */}
      <Box className="bg-light">
        <Container maxWidth="sm" sx={{ py: 8, textAlign: "center" }}>
          <Typography
            variant="h3"
            component="h2"
            gutterBottom
            sx={{ fontWeight: "bold", mb: 3 }}
          >
            The Mission
          </Typography>
          <Typography
            variant="body1"
            sx={{
              fontSize: "1.1rem",
              lineHeight: 1.8,
              color: "text.secondary",
            }}
          >
            Planting Life is a non-commercial, open-source passion project
            created to remove the barriers between people and their local
            environment. I believe that restoring our ecosystems should be easy
            for everyone, regardless of their gardening experience. By making it
            easy to find the right plants, we can all play a part in rebuilding
            a healthier planet.
          </Typography>
        </Container>
      </Box>

      {/* FAQ Section */}
      <Box>
        <Container maxWidth="md" sx={{ py: 8 }}>
          <Typography
            variant="h3"
            component="h2"
            textAlign="center"
            gutterBottom
            sx={{ fontWeight: "bold", mb: 6 }}
          >
            Frequently Asked Questions
          </Typography>
          <Box sx={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {[
              {
                q: "What is a native plant?",
                a: "Native plants are species that occurred naturally in your region long before human intervention. They have co-evolved with local wildlife, making them the best choice for a sustainable yard.",
              },
              {
                q: "Where do you get information on plants?",
                a: "Planting Life prioritizes data from trusted sources like USDA's PLANTS Database and Lady Bird Johnson's Wildflower.org. When data isn't available from trusted sources, AI-based tools fill in the blanks.",
              },
              {
                q: "Is this tool free?",
                a: "Yes, it is free of charge. There are no subscriptions, no ads, and no sign-ups required. My only goal is to get more native plants in the ground.",
              },
              {
                q: "Where can I buy these plants?",
                a: "Once you've entered your zipcode, use the 'Nurseries' tab in the app to find local businesses that specialize in native species.",
              },
            ].map((faq, index) => (
              <Paper
                key={index}
                elevation={0}
                sx={{
                  p: 3,
                  borderRadius: "16px",
                  border: "1px solid rgba(76, 175, 80, 0.2)",
                  backgroundColor: "#f1f8e9",
                  transition:
                    "transform 0.2s, box-shadow 0.2s, background-color 0.2s",
                  "&:hover": {
                    backgroundColor: "#e8f5e9",
                    borderColor: "rgba(76, 175, 80, 0.3)",
                  },
                }}
              >
                <Typography
                  variant="h6"
                  component="div"
                  gutterBottom
                  sx={{ fontWeight: "bold", mb: 1 }}
                >
                  {faq.q}
                </Typography>
                <Typography
                  variant="body1"
                  color="text.secondary"
                  sx={{ lineHeight: 1.6 }}
                >
                  {faq.a}
                </Typography>
              </Paper>
            ))}
          </Box>
        </Container>
      </Box>
    </Box>
  );
};

export default LandingPage;
