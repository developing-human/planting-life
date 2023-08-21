import { useState, useEffect } from "react";
import { useParams, useLocation } from "react-router-dom";

// tabs
import DiscoverTab from "../tabs/DiscoverTab";
import GardenTab from "../tabs/GardenTab";
import NurseryTab from "../tabs/NurseryTab";

// material ui & styling
import YardIcon from "@mui/icons-material/Yard";
import Search from "@mui/icons-material/Search";
import StorefrontIcon from "@mui/icons-material/Storefront";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import Badge from "@mui/material/Badge";
import Box from "@mui/material/Box";
import "./Home.css";

const Home = () => {
  const DISCOVER_TAB_INDEX = 0;
  const GARDEN_TAB_INDEX = 1;
  const NURSERY_TAB_INDEX = 2;
  const [searchCriteria, setSearchCriteria] = useState({ zip: "" });
  const [lastSearchedCriteria, setLastSearchedCriteria] = useState(null);
  const [plants, setPlants] = useState([]);
  const [selectedPlants, setSelectedPlants] = useState([]);
  const [nurseries, setNurseries] = useState([]);
  const [selectedTab, setSelectedTab] = useState(DISCOVER_TAB_INDEX);
  const [error, setError] = useState(null);

  const showTabs =
    selectedTab !== DISCOVER_TAB_INDEX || lastSearchedCriteria != null;

  const handleTabChange = (event, newValue) => {
    setSelectedTab(newValue);
  };

  const loadGarden = (id) => {
    fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens/${id}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error(
            `Error fetching /gardens, status: ${response.status}`
          );
        }

        return response.json();
      })
      .then((data) => {
        console.log(data);
        setSelectedPlants(data.plants);
      })
      .catch((error) => {
        console.error(error);
        setSelectedTab(DISCOVER_TAB_INDEX);
        setError(`Couldn't find the garden you're looking for 😞`);
      });
  };

  const { id } = useParams();
  const location = useLocation();
  useEffect(() => {
    const currentPath = location.pathname;

    let tab;
    switch (currentPath) {
      case "/":
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

  useEffect(() => {
    // Don't scroll if tabs aren't shown.  Without this, it scrolls down when
    // the page loads on small screens.
    if (!showTabs) {
      return;
    }

    const elementId =
      selectedTab === DISCOVER_TAB_INDEX ? "discover-cards" : "tab-container";
    const extraOffset = selectedTab === 0 ? -90 : 0;

    // Find the top of the tab container
    const element = document.getElementById(elementId);
    const elementPosition = element.getBoundingClientRect().top;

    // If its negative, its above the top of the viewport and we need to scroll
    // up to the top when changing tabs.
    const offsetPosition = elementPosition + window.pageYOffset + extraOffset;
    window.scrollTo({ top: offsetPosition });
  }, [selectedTab, showTabs]);

  /*
  const navigate = useNavigate();
  const onViewGardenClick = () => {
    fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        zipcode: lastSearchedCriteria.zip,
        shade: lastSearchedCriteria.shade,
        moisture: lastSearchedCriteria.moisture,
        plant_ids: selectedPlants.map((p) => p.id),
      }),
    })
      .then((response) => response.json())
      .then((data) => {
        //TODO: Pass nurseries, search criteria, region_name, zipcode, read_id
        navigate(`gardens/${data.write_id}`, {
          state: { plants: selectedPlants },
        });
      })
      //TODO: Better error handling?
      .catch((error) => console.error("Error:", error));
  };
  */

  return (
    <>
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
                disabled={selectedPlants.length === 0}
                icon={
                  <Badge badgeContent={selectedPlants.length} color="success">
                    <YardIcon />
                  </Badge>
                }
                label="MY GARDEN"
              />
              <Tab
                disabled={nurseries.length === 0}
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
            setSelectedPlants={setSelectedPlants}
            searchCriteria={searchCriteria}
            setSearchCriteria={setSearchCriteria}
            setLastSearchedCriteria={setLastSearchedCriteria}
            error={error}
            setError={setError}
          />
        </CustomTabPanel>
        <CustomTabPanel value={selectedTab} index={GARDEN_TAB_INDEX}>
          <GardenTab selectedPlants={selectedPlants} />
        </CustomTabPanel>
        <CustomTabPanel value={selectedTab} index={NURSERY_TAB_INDEX}>
          <NurseryTab nurseries={nurseries} />
        </CustomTabPanel>
      </div>
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
