import { Routes, Route } from "react-router-dom";

// pages
import Home from "./pages/Home";
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
          <Route path="/gardens/:id" element={<Home />} />
          <Route path="/g/:id" element={<Home />} />
        </Routes>
      </main>

      <Footer />
    </div>
  );
}

export default App;
