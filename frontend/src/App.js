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

  //TODO: To restore search criteria, those need to be elevated too

  //TODO: I need to make selected plants show as selected on returning

  //TODO: Selected plants should clear when searching again.

  return (
    <div className="App">
      <NavBar />

      <main>
        <Routes>
          <Route path="/" element={<Home plants={plants} 
                                         setPlants={setPlants} 
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
