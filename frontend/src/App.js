import React, { useEffect } from "react";
import "./App.css";

// pages
import Home from "./pages/Home/Home";

// utilities
import { getData } from "./utilities/openai-api";

function App() {
  return (
    <div className="App">
      <Home />
    </div>
  );
}

export default App;
