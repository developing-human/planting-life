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
  React.useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("active");
          }
        });
      },
      { threshold: 0.1 },
    );

    document.querySelectorAll(".reveal").forEach((el) => observer.observe(el));
    return () => observer.disconnect();
  }, []);

  return (
    <Box
      sx={{ bgcolor: "var(--color-cream)", color: "var(--color-deep-green)" }}
    >
      {/* Hero Section */}
      <Box
        id="hero-container"
        sx={{
          backgroundImage: "url('/hero-caterpillar.jpg')",
          backgroundSize: "cover",
          backgroundPosition: "center",
          minHeight: "75vh",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          paddingY: { xs: 1, md: 8 },
          position: "relative",
          width: "100%",
          zIndex: 5,
        }}
      >
        <Container
          maxWidth="md"
          sx={{ px: 3, position: "relative", zIndex: 2 }}
        >
          <Box
            sx={{
              backgroundColor: "rgba(255, 255, 255, 0.08)",
              backdropFilter: "blur(8px)",
              WebkitBackdropFilter: "blur(8px)",
              borderRadius: "40px",
              padding: { xs: 3, sm: 6 },
              textAlign: "center",
              color: "white",
              border: "1px solid rgba(255, 255, 255, 0.2)",
              boxShadow: "0 20px 50px rgba(0,0,0,0.2)",
            }}
          >
            <Typography
              variant="h2"
              component="h1"
              sx={{
                fontFamily: "var(--font-heading)",
                fontWeight: "bold",
                fontSize: { xs: "2rem", sm: "3rem", md: "4.5rem" },
                lineHeight: 1.1,
                mb: 3,
                color: "white",
              }}
            >
              Find native plants for{" "}
              <em style={{ fontFamily: "var(--font-heading)" }}>your</em> garden
            </Typography>
            <Typography
              variant="h5"
              sx={{
                fontFamily: "var(--font-body)",
                fontSize: { xs: "1rem", sm: "1.2rem", md: "1.5rem" },
                mb: { xs: 0, md: 2 },
                fontWeight: 300,
                color: "rgba(255, 255, 255, 0.9)",
                maxWidth: "600px",
                mx: "auto",
              }}
            >
              Create a thriving yard that supports local wildlife.
            </Typography>
            <Box
              sx={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
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
                {[
                  {
                    icon: <Search />,
                    text: "Find plants that fit your zip code & growing conditions",
                  },
                  {
                    icon: <ShareIcon />,
                    text: "Create a garden plan to share with friends or nurseries",
                  },
                  {
                    icon: <LocationOnIcon />,
                    text: "Find nurseries in your area that specialize in native plants",
                  },
                ].map((item, i) => (
                  <ListItem key={i} disableGutters sx={{ py: 0.25 }}>
                    <ListItemIcon
                      sx={{ minWidth: 40, color: "var(--color-sage)" }}
                    >
                      {item.icon}
                    </ListItemIcon>
                    <ListItemText
                      primary={item.text}
                      primaryTypographyProps={{
                        variant: "body1",
                        sx: { color: "white", fontWeight: 300 },
                      }}
                    />
                  </ListItem>
                ))}
              </List>
              <Box sx={{ textAlign: "center" }}>
                <Button
                  className="btn-organic"
                  variant="contained"
                  size="large"
                  onClick={onStartGarden}
                  sx={{
                    px: { xs: 3, md: 6 },
                    pt: { xs: 1, md: 2 },
                    pb: { xs: 1, md: 1.25 },
                    fontSize: "1.2rem",
                    whiteSpace: "nowrap",
                  }}
                >
                  Start Your Garden
                </Button>
                <Typography
                  variant="caption"
                  display="block"
                  sx={{
                    mt: 2,
                    color: "rgba(255, 255, 255, 0.7)",
                    fontSize: "0.9rem",
                    fontWeight: 400,
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
      <Box
        className="bg-light"
        sx={{ py: { xs: 5, md: 8 }, overflow: "hidden" }}
      >
        <Container maxWidth="lg">
          <Box sx={{ textAlign: "center", mb: 8, className: "reveal" }}>
            <Typography
              variant="h3"
              component="h2"
              sx={{
                fontFamily: "var(--font-heading)",
                fontWeight: "bold",
                mb: 3,
                fontSize: { xs: "2rem", md: "3rem" },
                color: "var(--color-forest)",
              }}
            >
              Why Plant Native?
            </Typography>
            <Typography
              variant="h6"
              sx={{
                mb: 5,
                color: "text.secondary",
                lineHeight: 1.6,
                maxWidth: "700px",
                mx: "auto",
                fontWeight: 300,
                fontFamily: "var(--font-body)",
              }}
            >
              Planting native is about more than just aesthetics, it's about
              supporting our local ecosystem. Native plants provide the
              essential food and shelter that local butterflies, bees, and birds
              need to thrive.
            </Typography>
          </Box>

          <Grid container spacing={8} alignItems="center">
            {[
              {
                title: "Support Pollinators",
                text: "Native plants provide the exact nectar and host leaves that local butterflies and bees need to survive.",
                img: "https://images.unsplash.com/photo-1565041222041-db9ebdda950a?q=80&w=1473&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D",
                reverse: false,
              },
              {
                title: "Built for Your Backyard",
                text: "Because they evolved in your region, native plants are naturally adapted to your climate, which means they generally need less water and fewer fertilizers.",
                img: "https://images.unsplash.com/photo-1707021970546-abd3380f5a8f?q=80&w=1470&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D",
                reverse: true,
              },
              {
                title: "Restore the Balance",
                text: "Even a small garden can act as a vital stepping stone for wildlife, helping to bring nature back into our neighborhoods.",
                img: "https://images.unsplash.com/photo-1599063906749-e3c28e0d5a13?q=80&w=1480&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D",
                reverse: false,
              },
            ].map((item, index) => (
              <Grid item xs={12} md={4} key={index} className="reveal">
                <Box
                  sx={{
                    display: "flex",
                    flexDirection: {
                      xs: "column",
                      md: item.reverse ? "column-reverse" : "column",
                    },
                    gap: 3,
                    textAlign: "center",
                  }}
                >
                  <Box
                    className="organic-shape"
                    sx={{
                      width: "100%",
                      height: "300px",
                      backgroundImage: `url(${item.img})`,
                      backgroundSize: "cover",
                      backgroundPosition: "center",
                      boxShadow: "0 20px 40px rgba(0,0,0,0.1)",
                      transition: "transform 0.3s ease",
                      "&:hover": { transform: "scale(1.02)" },
                    }}
                  />
                  <Box>
                    <Typography
                      variant="h5"
                      component="h3"
                      sx={{
                        fontFamily: "var(--font-heading)",
                        fontWeight: "bold",
                        mb: 2,
                        color: "var(--color-forest)",
                      }}
                    >
                      {item.title}
                    </Typography>
                    <Typography
                      variant="body1"
                      color="text.secondary"
                      sx={{ lineHeight: 1.7, fontWeight: 300 }}
                    >
                      {item.text}
                    </Typography>
                  </Box>
                </Box>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Gardening Made Simple Section */}
      <Box sx={{ py: { xs: 8, md: 12 }, bgcolor: "var(--color-cream)" }}>
        <Container maxWidth="md">
          <Box sx={{ textAlign: "center", mb: 10, className: "reveal" }}>
            <Typography
              variant="h3"
              component="h2"
              sx={{
                fontFamily: "var(--font-heading)",
                fontWeight: "bold",
                mb: 3,
                fontSize: { xs: "2rem", md: "3rem" },
                color: "var(--color-forest)",
              }}
            >
              Native Gardening Made Simple
            </Typography>
          </Box>

          <Box sx={{ display: "flex", flexDirection: "column", gap: 6 }}>
            {[
              {
                step: "01",
                title: "Describe your space",
                text: "Enter your zip code and a few details about your yard, like how much sun it gets and how wet it is.",
                color: "var(--color-sage)",
              },
              {
                step: "02",
                title: "Find your matches",
                text: "Get a list of plants that are a great fit for your garden.",
                color: "var(--color-ochre)",
              },
              {
                step: "03",
                title: "Get the details",
                text: "Dive into the details, from bloom times to whether the plants are deer-resistant.",
                color: "var(--color-terracotta)",
              },
              {
                step: "04",
                title: "Bring it to life",
                text: "Save your favorites to a garden plan that you can keep or share with a local nursery.",
                color: "var(--color-forest)",
              },
            ].map((item, index) => (
              <Box
                key={index}
                className="reveal"
                sx={{
                  display: "flex",
                  alignItems: "center",
                  gap: 4,
                  p: 4,
                  borderRadius: "32px",
                  bgcolor: "white",
                  boxShadow: "0 10px 30px rgba(0,0,0,0.03)",
                  border: "1px solid rgba(0,0,0,0.05)",
                  flexDirection: { xs: "column", sm: "row" },
                  textAlign: { xs: "center", sm: "left" },
                }}
              >
                <Typography
                  variant="h2"
                  sx={{
                    fontWeight: "bold",
                    fontSize: "4rem",
                    opacity: 0.2,
                    color: item.color,
                    lineHeight: 1,
                    minWidth: "80px",
                  }}
                >
                  {item.step}
                </Typography>
                <Box>
                  <Typography
                    variant="h5"
                    sx={{
                      fontFamily: "var(--font-heading)",
                      fontWeight: "bold",
                      mb: 1,
                      color: "var(--color-deep-green)",
                    }}
                  >
                    {item.title}
                  </Typography>
                  <Typography
                    variant="body1"
                    color="text.secondary"
                    sx={{ lineHeight: 1.6, fontWeight: 300 }}
                  >
                    {item.text}
                  </Typography>
                </Box>
              </Box>
            ))}
          </Box>
        </Container>
      </Box>

      {/* Mission Section */}
      <Box
        sx={{
          py: { xs: 12, md: 16 },
          bgcolor: "var(--color-deep-green)",
          color: "var(--color-cream)",
          position: "relative",
          overflow: "hidden",
        }}
      >
        <Box
          sx={{
            position: "absolute",
            top: "-10%",
            right: "-10%",
            width: "60%",
            height: "60%",
            bgcolor: "rgba(183, 183, 164, 0.1)",
            className: "organic-shape-alt",
            zIndex: 1,
          }}
        />
        <Container
          maxWidth="sm"
          sx={{ position: "relative", zIndex: 2, textAlign: "center" }}
        >
          <Box className="reveal">
            <Typography
              variant="h3"
              component="h2"
              gutterBottom
              sx={{
                fontFamily: "var(--font-heading)",
                fontWeight: "bold",
                mb: 4,
                color: "var(--color-sage)",
              }}
            >
              The Mission
            </Typography>
            <Typography
              variant="body1"
              sx={{
                fontSize: "1.2rem",
                lineHeight: 1.8,
                color: "rgba(253, 252, 240, 0.8)",
                fontWeight: 300,
                fontFamily: "var(--font-body)",
              }}
            >
              Planting Life is a non-commercial, open-source passion project
              created to remove the barriers between people and their local
              environment. I believe that restoring our ecosystems should be
              easy for everyone, regardless of their gardening experience. By
              making it easy to find the right plants, we can all play a part in
              rebuilding a healthier planet.
            </Typography>
          </Box>
        </Container>
      </Box>

      {/* FAQ Section */}
      <Box sx={{ py: { xs: 8, md: 12 }, bgcolor: "var(--color-cream)" }}>
        <Container maxWidth="md">
          <Box sx={{ textAlign: "center", mb: 10, className: "reveal" }}>
            <Typography
              variant="h3"
              component="h2"
              sx={{
                fontFamily: "var(--font-heading)",
                fontWeight: "bold",
                mb: 2,
                fontSize: { xs: "2rem", md: "3rem" },
                color: "var(--color-forest)",
              }}
            >
              Frequently Asked Questions
            </Typography>
            <Box
              sx={{
                width: "60px",
                height: "4px",
                bgcolor: "var(--color-ochre)",
                mx: "auto",
                borderRadius: "2px",
              }}
            />
          </Box>
          <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
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
              <Box
                key={index}
                className="reveal"
                sx={{
                  p: 3,
                  borderRadius: "24px",
                  borderBottom: "1px solid rgba(27, 67, 50, 0.1)",
                  transition: "all 0.3s ease",
                  "&:hover": {
                    bgcolor: "rgba(255, 255, 255, 0.5)",
                    transform: "translateX(10px)",
                  },
                }}
              >
                <Typography
                  variant="h6"
                  component="div"
                  sx={{
                    fontFamily: "var(--font-heading)",
                    fontWeight: "bold",
                    mb: 1,
                    color: "var(--color-forest)",
                  }}
                >
                  {faq.q}
                </Typography>
                <Typography
                  variant="body1"
                  color="text.secondary"
                  sx={{ lineHeight: 1.6, fontWeight: 300 }}
                >
                  {faq.a}
                </Typography>
              </Box>
            ))}
          </Box>
        </Container>
      </Box>
    </Box>
  );
};

export default LandingPage;
