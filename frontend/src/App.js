import { Routes, Route } from "react-router-dom";

// pages
import Home from "./pages/Home";
import Garden from "./pages/Garden";
import NavBar from "./components/NavBar";
import Footer from "./components/Footer";

// styling
import "./App.css";

function App() {
  return (
    <div className="App">
      <NavBar />

      <main>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/gardens" element={<Garden />} />
          <Route path="/gardens/:id" element={<Garden />} />
        </Routes>
      </main>

      <Footer />
    </div>
  );
}

export default App;
