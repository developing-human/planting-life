import { Routes, Route } from "react-router-dom";

// pages
import Home from "./pages/Home/Home";

// styling
import "./App.css";

function App() {
  return (
    <div className="App">
      <Routes>
        <Route path="/" element={ <Home /> } />
      </Routes>
    </div>
  );
}

export default App;
