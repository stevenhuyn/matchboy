// import { createSignal, onMount } from "solid-js";
// import solidLogo from "./assets/solid.svg";
// import viteLogo from "/vite.svg";
// import init, { greet, connect, send_message } from "matchlib";
import { Route, Routes } from "@solidjs/router";
import "./App.css";
import { LandingPage } from "./pages/LandingPage";

const App = () => {
  return (
    <Routes>
      <Route path="/*" component={LandingPage}></Route>
    </Routes>
  );
};


export default App;
