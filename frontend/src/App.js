import { useState } from "react"
import { Routes, Route } from "react-router-dom";

// pages
import Home from "./pages/Home/Home";
import Garden from "./pages/Garden/Garden";
import NavBar from "./components/NavBar/NavBar";
import Footer from "./components/Footer/Footer";

// styling
import "./App.css";

function App() {

  // This state lives at the App level so it can be restored when
  // navigating back from Garden to Home.
  const [plants, setPlants] = useState([]);
  const [nurseries, setNurseries] = useState([]);
  const [selectedPlants, setSelectedPlants] = useState([]);
  const [maxPlantsToDisplay, setMaxPlantsToDisplay] = useState(12);
  const [searchCriteria, setSearchCriteria] = useState({});

  //TODO: Not a common scenario... but it could be worth differentiating
  //      between "search criteria" (what is currently in the box) and
  //      "searched criteria" (what was in the box for the last search)

  return (
    <div className="App">
      <NavBar />

      <main>
        <Routes>
          <Route path="/" element={<Home plants={plants} 
                                         setPlants={setPlants} 
                                         searchCriteria={searchCriteria} 
                                         setSearchCriteria={setSearchCriteria} 
                                         nurseries={nurseries} 
                                         setNurseries={setNurseries} 
                                         selectedPlants={selectedPlants} 
                                         setSelectedPlants={setSelectedPlants} 
                                         maxPlantsToDisplay={maxPlantsToDisplay} 
                                         setMaxPlantsToDisplay={setMaxPlantsToDisplay} />

          } />
          <Route path="/garden" element={<Garden />} />
        </Routes>
      </main>

      <Footer />
    </div>
  );
}

export default App;
