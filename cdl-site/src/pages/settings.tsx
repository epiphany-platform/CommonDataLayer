import { FunctionalComponent, h } from "preact";
import { useContext } from "preact/hooks";

import CdlContext from "../context";

const Settings: FunctionalComponent = () => {
    const { darkMode, toggleDarkMode } = useContext(CdlContext);

    return (
        <div class="container container-small">
            <div class="row">
                <div class="col align-center">
                    <h2>Settings</h2>
                    <br />
                    <p>
                        <button onClick={toggleDarkMode}>
                            Your colorscheme is {darkMode ? "Dark Mode" : "Light Mode"}
                        </button>
                    </p>
                </div>
            </div>
        </div>
    );
};

export default Settings;
