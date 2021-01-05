import { createContext } from "preact";

import { RemoteData, Schema, notLoaded } from "./models";

interface ContextProps {
    darkMode: boolean;
    toggleDarkMode: () => void;
    schemas: RemoteData<Schema[]>;
    setSchemas: (schemas: RemoteData<Schema[]>) => void;
}

const CdlContext = createContext<ContextProps>({
    darkMode: false,
    toggleDarkMode: () => { },
    schemas: notLoaded,
    setSchemas: () => { }
});

export default CdlContext;
