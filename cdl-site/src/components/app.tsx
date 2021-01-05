import { FunctionalComponent, h } from "preact";
import { Route, Router } from "preact-router";
import { useState, useEffect, useCallback, useContext } from "preact/hooks";

import Home from "../pages/home";
import Settings from "../pages/settings";
import Schemas from "../pages/schemas";
import NotFound from "../pages/not-found";
import Header from "./header";
import CdlContext from "../context";
import { Schema, notLoaded, RemoteData } from "../models";

const App: FunctionalComponent = () => {
    const [menuOpen, toggleMenu] = useState(false);
    const [darkMode, setDarkMode] = useState(false);
    const [schemas, setSchemas] = useState<RemoteData<Schema[]>>(notLoaded);

    const toggleDarkMode = useCallback(() => {
        console.log(`dark mode: ${darkMode}`);
        setDarkMode(!darkMode);
        localStorage.setItem("dark-mode", JSON.stringify(!darkMode));
    }, [darkMode, setDarkMode]);

    useEffect(() => {
        const savedDarkMode = JSON.parse(localStorage.getItem("dark-mode") || "false");
        setDarkMode(savedDarkMode);
    }, [setDarkMode]);

    return (
        <CdlContext.Provider
            value={{
                darkMode,
                toggleDarkMode,
                schemas,
                setSchemas
            }}
        >
            <div id="app" class={darkMode ? "dark-mode" : ""} onClick={() => toggleMenu(false)}>
                <Header menuOpen={menuOpen} toggleMenu={toggleMenu} />
                <Router>
                    <Route path="/" component={Home} />
                    <Route path="/settings" component={Settings} />
                    <Route path="/schemas" component={Schemas} schemaId={null} version={null} creating={false} />
                    <Route path="/schemas/new" component={Schemas} schemaId={null} version={null} creating={true} />
                    <Route path="/schemas/:schemaId" component={Schemas} version={null} creating={false} />
                    <Route path="/schemas/:schemaId/:version" component={Schemas} creating={false} />
                    <NotFound default />
                </Router>
            </div>
        </CdlContext.Provider>
    );
};

export default App;
