import { useState, useEffect } from "react";
// components
import ConditionsForm from "../../components/ConditionsForm/ConditionsForm";
import IntroAccordion from "../../components/IntroAccordion/IntroAccordion";
import Spinner from "../../components/Spinner/Spinner";
import PlantCard from "../../components/PlantCard/PlantCard";
import Nursery from "../../components/Nursery/Nursery";

// material ui & styling
import YardIcon from '@mui/icons-material/Yard'
import Search from '@mui/icons-material/Search'
import StorefrontIcon from '@mui/icons-material/Storefront'
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Badge from '@mui/material/Badge';
import Box from '@mui/material/Box';

import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import "./Home.css";

import { useNavigate } from "react-router-dom";

const Home = ({
  plants, setPlants, 
  nurseries, setNurseries, 
  selectedPlants, setSelectedPlants, 
  maxPlantsToDisplay, setMaxPlantsToDisplay,
  searchCriteria, setSearchCriteria
}) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [infoMessage, setInfoMessage] = useState(null);
  const [expanded, setExpanded] = useState('welcome');
  
  const showMoreButton = plants.length >= maxPlantsToDisplay;
  const showSpinner = loading && plants.length < maxPlantsToDisplay;
  const showSurvey = loading || plants.length > 0;
  const showTabs = loading || plants.length > 0 || nurseries.length > 0;

  const [selectedTab, setSelectedTab] = useState(0);

  const plantsWithImages = plants.filter((plant) => plant.image);

  const onMoreClick = () => {
    setMaxPlantsToDisplay((oldMax) => oldMax + 12);
  };

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setSelectedTab(newValue);
  };

  useEffect(() => {
    const elementId = selectedTab === 0 ? 'discover-cards' : 'tab-container';
    const extraOffset = selectedTab === 0 ? -90 : 0;
    console.log(selectedTab);
    console.log(elementId);
    // Find the top of the tab container
    const element = document.getElementById(elementId);
    const elementPosition = element.getBoundingClientRect().top;

    // If its negative, its above the top of the viewport and we need to scroll
    // up to the top when changing tabs.
    const offsetPosition = elementPosition + window.pageYOffset + extraOffset;
    window.scrollTo({top: offsetPosition});
  }, [selectedTab])

  const navigate = useNavigate();
  const onViewGardenClick = () => {
    fetch(`${process.env.REACT_APP_URL_PREFIX}/gardens`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        zipcode: searchCriteria.zip,
        shade: searchCriteria.shade,
        moisture: searchCriteria.moisture,
        plant_ids: selectedPlants.map((p) => p.id)
      })
    })
    .then(response => response.json())
    .then(data => {
      //TODO: Pass nurseries, search criteria, region_name, zipcode, read_id
      navigate(`gardens/${data.write_id}`, {state: {plants: selectedPlants}});
    })
    //TODO: Better error handling?
    .catch(error => console.error("Error:", error));
  };

  return (
    <>
        <div id="tab-container">
          { showTabs ? 
            <div style={{position: "sticky", 
                         top: 0, 
                         backgroundColor: "white", 
                         paddingTop: "2px",
                         zIndex: 5}}>
              <Tabs value={selectedTab} 
                    onChange={handleTabChange} 
                    aria-label="icon label tabs example" 
                    centered 
                    sx={{maxWidth: "1000px", margin: "auto"}}
                    variant="fullWidth">
                <Tab icon={<Search />} label="DISCOVER" />
                <Tab disabled={selectedPlants.length === 0} icon={<Badge badgeContent={selectedPlants.length} color="success">
                             <YardIcon />
                           </Badge>} 
                     label="MY GARDEN" />
                <Tab disabled={nurseries.length === 0} icon={<Badge badgeContent={nurseries.length} color="success">
                             <StorefrontIcon />
                           </Badge>} 
                     label="Nurseries" />
              </Tabs>
            </div> : null
          }
          <CustomTabPanel value={selectedTab} index={0}>
      <ConditionsForm searchCriteria={searchCriteria}
                      setSearchCriteria={setSearchCriteria} 
                      setPlants={setPlants} 
                      setNurseries={setNurseries} 
                      setLoading={setLoading} 
                      setError={setError} 
                      setInfoMessage={setInfoMessage} 
                      setExpanded={setExpanded}
                      setMaxPlantsToDisplay={setMaxPlantsToDisplay}
                      setSelectedPlants={setSelectedPlants}
                      plants={plants}/>

      <div className="accordion-container">
        <IntroAccordion expanded={expanded} setExpanded={setExpanded}/>
      </div>

      {
        error || infoMessage ?
          <div className="alert-container">
            {error ? <Alert severity="error">{error}</Alert> : null}
            {infoMessage ? <Alert severity="info">{infoMessage}</Alert> : null}
          </div>
          : null
      }

      <div className="alert-container" id="top-survey-alert">
      {
        showSurvey ?
        <Alert severity="info">Help decide how Planting Life grows by <a href="https://docs.google.com/forms/d/e/1FAIpQLSfN9W9GusLRo5rIX3yENrBLKcNIu3y9BQpdRwOnCYYvTSX3zA/viewform?usp=sf_link" target="_blank" rel="noreferrer">sharing your thoughts</a>.</Alert>
        : null
      }

      </div>


            <section className="card-container" id="discover-cards">
              {plantsWithImages.slice(0, maxPlantsToDisplay).map((plant, index) => (
                <PlantCard plant={plant} key={plant.id} setSelectedPlants={setSelectedPlants} setPlants={setPlants}/>
             ))}
              {showSpinner ? <Spinner /> : null}
              
            </section>
          
            <div className="button-container">
                {showMoreButton &&
                    <Button className="more-button" 
                            type="submit" 
                            onClick={onMoreClick}>Load More</Button>
                }
            </div>
          </CustomTabPanel>
          <CustomTabPanel value={selectedTab} index={1}>
            <section className="card-container">
              {selectedPlants.map((plant, index) => (
                <PlantCard plant={plant} 
                           key={plant.id}
                           setSelectedPlants={setSelectedPlants} 
                           showAddButton={false}
                           setPlants={setPlants}/>
             ))}
              {showSpinner ? <Spinner /> : null}
              
            </section>
          </CustomTabPanel>
          <CustomTabPanel value={selectedTab} index={2}>
            <section className="card-container">
              {nurseries.map((nursery, index) => (
                <Nursery nursery={nursery} key={index} />
              ))}
            </section>
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
      {value === index && (
        <Box sx={{ p: 3 }}>
          {children}
        </Box>
      )}
    </div>
  );
}


export default Home;
