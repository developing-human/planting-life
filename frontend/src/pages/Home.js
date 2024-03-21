import { useState, useEffect } from "react";
import { useParams, useLocation } from "react-router-dom";

// tabs
import DiscoverTab from "../tabs/DiscoverTab";
import GardenTab from "../tabs/GardenTab";
import NurseryTab from "../tabs/NurseryTab";

import { getGarden, saveGarden } from "../utilities/garden-api";

// material ui & styling
import YardIcon from "@mui/icons-material/Yard";
import Search from "@mui/icons-material/Search";
import StorefrontIcon from "@mui/icons-material/Storefront";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import Badge from "@mui/material/Badge";
import Box from "@mui/material/Box";
import "./Home.css";
import { getPlants } from "../utilities/plant-api";
import { getNurseries } from "../utilities/nursery-api";
import { Alert, Button, Paper, Snackbar } from "@mui/material";

const Home = () => {
  const DISCOVER_TAB_INDEX = 0;
  const GARDEN_TAB_INDEX = 1;
  const NURSERY_TAB_INDEX = 2;
  const [searchCriteria, setSearchCriteria] = useState({ zip: "" });
  const [lastSearchedCriteria, setLastSearchedCriteria] = useState(null);
  const [plants, setPlants] = useState([]);
  const [nurseries, setNurseries] = useState([]);
  const [selectedTab, setSelectedTab] = useState(DISCOVER_TAB_INDEX);
  const [error, setError] = useState(null);
  const [garden, setGarden] = useState({ plants: [], name: "" });
  const [isCopyGardenMessageOpen, setIsCopyGardenMessageOpen] = useState(false);
  const [nurserySearchZip, setNurserySearchZip] = useState("");
  const location = useLocation();
  const [showHero, setShowHero] = useState(location.pathname === "/");

  const showTabs =
    selectedTab !== DISCOVER_TAB_INDEX ||
    plants.length > 0 ||
    garden.plants.length > 0;

  const handleTabChange = (event, newValue) => {
    setSelectedTab(newValue);

    // Scroll to top when switching tabs
    window.scrollTo({ top: 0, behavior: "auto" });
  };

  const loadGarden = (id) => {
    getGarden(
      id,
      (fetchedGarden) => {
        // Everything in the garden is selected by default
        fetchedGarden.plants.forEach((plant) => {
          plant.selected = true;
        });

        setGarden(fetchedGarden);

        const gardenSearchCriteria = {
          zip: fetchedGarden.zipcode,
          shade: fetchedGarden.shade,
          moisture: fetchedGarden.moisture,
        };

        // Populate the Discover tab's search criteria based on the criteria
        // that were used to build the loaded garden
        setSearchCriteria(gardenSearchCriteria);

        // Populate lastSearchedCriteria since plants will be loaded as though
        // they were searched for
        setLastSearchedCriteria(gardenSearchCriteria);

        // Populate the discover tab based on those criteria, too
        getPlants(
          gardenSearchCriteria,
          setPlants,
          setError,
          fetchedGarden.plants //TODO: Not handled yet after refactor
        );

        // Populate the Nurseries tab using the zipcode from the garden
        getNurseries(fetchedGarden.zipcode, setNurseries);
      },
      (error) => {
        console.error(error);
        setSelectedTab(DISCOVER_TAB_INDEX);
        setError(`Couldn't find the garden you're looking for 😞`);
      }
    );
  };

  const onNewGarden = () => {
    // Clear the garden.  Its a new garden.
    setGarden({ plants: [] });

    // Set all plants as unselected
    setPlants((prevPlants) => {
      return prevPlants.map((plant) => ({
        ...plant,
        selected: false,
      }));
    });

    // Since we're keeping search results, keep the last searched criteria
    setLastSearchedCriteria(searchCriteria);
  };

  const showCopyGardenMessage = () => {
    setIsCopyGardenMessageOpen(true);
  };

  const closeCopyGardenMessage = (event, reason) => {
    if (reason === "clickaway") {
      return;
    }

    setIsCopyGardenMessageOpen(false);
  };

  const handleStartGardenClicked = () => {
    setShowHero(false);
    document.getElementById("zip").focus();
  };

  // When the page is loaded, process the URL path and load data / switch tabs
  const { id } = useParams();
  useEffect(() => {
    const currentPath = location.pathname;

    let tab;
    switch (currentPath) {
      case "/":
        tab = DISCOVER_TAB_INDEX;
        break;
      case "/gardens":
        tab = DISCOVER_TAB_INDEX;
        break;
      case `/g/${id}`:
      case `/gardens/${id}`:
        tab = GARDEN_TAB_INDEX;
        loadGarden(id);
        break;
      default:
        tab = DISCOVER_TAB_INDEX;
    }

    setSelectedTab(tab);
  }, [id, location, setSelectedTab]);

  // When ids change, update the page's url.  Prioritize write url over read.
  useEffect(() => {
    if (showHero === true) {
      return; // Don't set URL if hero div is shown
    }

    const id = garden.write_id || garden.read_id;
    const url = id ? `/gardens/${id}` : "/gardens";
    window.history.replaceState(null, garden.name, url);
  }, [garden.read_id, garden.write_id, garden.name, showHero]);

  // When a Garden is updated, save it.
  useEffect(() => {
    // By checking a needsSave flag, we avoid saving for garden updates which
    // didn't intend to cause a save (i.e. loading from GET /gardens).
    if (garden.needsSave) {
      if (garden.read_id && !garden.write_id) {
        showCopyGardenMessage();
      }

      saveGarden(garden, setGarden, lastSearchedCriteria, (error) => {
        console.error(error);
      });

      setGarden((prevGarden) => {
        return { ...prevGarden, needsSave: false };
      });
    }
  }, [garden, lastSearchedCriteria]);

  // When search criteria are updated, update the nursery search zip as well.
  useEffect(() => {
    setNurserySearchZip(lastSearchedCriteria?.zip || "");
  }, [lastSearchedCriteria]);

  return (
    <>
      {showHero ? (
        <Paper id="hero-container">
          <div id="hero-contents">
            <h1>
              Discover native plants
              <br /> & plan your garden
            </h1>
            <h5>
              <b>Search for native plants</b> by zipcode & growing conditions
              <br />
              <b>Plan your garden</b> to share with friends or a nursery <br />
              <b>Find a nursery</b> near you which focuses on native plants
            </h5>
            <Button
              variant="contained"
              color="success"
              onClick={handleStartGardenClicked}
            >
              Start Your Garden
            </Button>
            <div id="free-no-signup">(Free, no signup)</div>
          </div>
        </Paper>
      ) : null}
      <div id="tab-container">
        {showTabs ? (
          <Box
            sx={{
              position: "sticky",
              top: 0,
              backgroundColor: "white",
              paddingTop: "2px",
              borderBottom: 1,
              borderColor: "divider",
              zIndex: 5,
            }}
          >
            <Tabs
              value={selectedTab}
              onChange={handleTabChange}
              aria-label="icon label tabs example"
              centered
              sx={{ maxWidth: "1000px", margin: "auto" }}
              variant="fullWidth"
            >
              <Tab icon={<Search />} label="DISCOVER" />
              <Tab
                icon={
                  <Badge badgeContent={garden.plants.length} color="success">
                    <YardIcon />
                  </Badge>
                }
                label="MY GARDEN"
              />
              <Tab
                icon={
                  <Badge badgeContent={nurseries.length} color="success">
                    <StorefrontIcon />
                  </Badge>
                }
                label="Nurseries"
              />
            </Tabs>
          </Box>
        ) : null}
        <CustomTabPanel value={selectedTab} index={DISCOVER_TAB_INDEX}>
          <DiscoverTab
            plants={plants}
            setPlants={setPlants}
            setNurseries={setNurseries}
            garden={garden}
            setGarden={setGarden}
            searchCriteria={searchCriteria}
            setSearchCriteria={setSearchCriteria}
            setLastSearchedCriteria={setLastSearchedCriteria}
            error={error}
            setError={setError}
          />
        </CustomTabPanel>
        <CustomTabPanel value={selectedTab} index={GARDEN_TAB_INDEX}>
          <GardenTab
            garden={garden}
            setGarden={setGarden}
            onNewGarden={onNewGarden}
            setPlants={setPlants}
          />
        </CustomTabPanel>
        <CustomTabPanel value={selectedTab} index={NURSERY_TAB_INDEX}>
          <NurseryTab
            nurseries={nurseries}
            setNurseries={setNurseries}
            zip={nurserySearchZip}
            setZip={setNurserySearchZip}
          />
        </CustomTabPanel>
      </div>
      <Snackbar
        open={isCopyGardenMessageOpen}
        autoHideDuration={4000}
        onClose={closeCopyGardenMessage}
      >
        <Alert
          onClose={closeCopyGardenMessage}
          severity="success"
          variant="filled"
        >
          Created your copy of this garden
        </Alert>
      </Snackbar>
    </>
  );
};

function CustomTabPanel(props) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`simple-tabpanel-${index}`}
      aria-labelledby={`simple-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

export default Home;
