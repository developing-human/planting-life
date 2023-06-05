import { Routes, Route } from "react-router-dom";

// pages
import Home from "./pages/Home/Home";
import NavBar from "./components/NavBar/NavBar";

// styling
import "./App.css";

function App() {
  return (
    <div className="App">
      <NavBar />

      <Routes className="content">
        <Route path="/" element={ <Home /> } />
      </Routes>
    </div>
  );
}

export default App;
