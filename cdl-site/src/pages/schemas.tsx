import { FunctionalComponent, h } from "preact";
import { Link } from "preact-router/match";
import { useState, useEffect, useContext } from "preact/hooks";

import { Schema, NewSchema, SchemaVersion, loaded, loading } from "../models";
import { allSchemas } from "../sample-data";
import CdlContext from "../context";

interface SchemasProps {
    schemaId: string | null;
    version: string | null;
    creating: boolean;
}

const Schemas: FunctionalComponent<SchemasProps> = (props) => {
    const { schemas, setSchemas } = useContext(CdlContext);

    useEffect(() => {
        setTimeout(() => {
            setSchemas(loaded(allSchemas));
        }, 1000);
    }, [setSchemas]);

    if (!(schemas.status === "loaded")) {
        return (<SchemasAreLoading />);
    } else if (schemas.data.length === 0) {
        return (<NoSchemas />);
    }

    const schema = schemas.data.find((schema) => schema.id === props.schemaId);

    return (
        <div class="container">
            <div class="row">
                <div class="col align-center">
                    <h2>Schemas</h2>
                </div>
            </div>
            <div class="row">
                <SchemaSidebar
                    schemas={schemas.data}
                    selectedId={props.schemaId}
                    createNewSchema={() => { }}
                />
                {schema ? (
                    <SchemaOverview schema={schema} version={props.version} />
                ) : (
                        <div
                            class="col-sm-9 align-center"
                            style={{ margin: "auto" }}
                        >
                            <p>Please select a schema from the left.</p>
                        </div>
                    )}
            </div>
        </div>
    );
};

const SchemasAreLoading: FunctionalComponent = () => (
    <div class="container container-small">
        <div class="row">
            <div class="col align-center">
                <h2>Your Schemas (are loading...)</h2>
                <div class="progress-bar striped animated">
                    <span class="progress-bar-green" style="width: 100%;"></span>
                </div>
            </div>
        </div>
    </div>
);

const NoSchemas: FunctionalComponent = () => (
    <div class="container container-small">
        <div class="row align-center">
            <div class="col align-center">
                <h2>Schemas</h2>
                <p>You have no schemas.</p>
                <p>
                    <Link href="/schemas/new">
                        <button>Create a Schema</button>
                    </Link>
                </p>
            </div>
        </div>
    </div>
);

interface SchemaSidebarProps {
    schemas: Schema[];
    selectedId: string | null;
    createNewSchema: () => void;
}

const SchemaSidebar: FunctionalComponent<SchemaSidebarProps> = ({
    schemas,
    selectedId,
    createNewSchema
}) => (
    <div class="col-sm-3">
        <div class="sidebar sidebar-left align-right">
            <h3 class="sidebar-category">Your Schemas</h3>
            <ul class="sidebar-links">
                {schemas.map((schema) => (
                    <li>
                        <a
                            class={schema.id === selectedId ? "active" : ""}
                            href={`/schemas/${schema.id}/`}
                            title={schema.id}
                        >
                            {schema.name}
                        </a>
                    </li>
                ))}
                <br />
                <li>
                    <button onClick={createNewSchema}>
                        Add New Schema
                    </button>
                </li>
            </ul>
        </div>
    </div>
);

interface SchemaOverviewProps {
    schema: Schema;
    version: string | null;
}

const SchemaOverview: FunctionalComponent<SchemaOverviewProps> = ({ schema, version }) => (
    <div class="col-sm-9 align-center">
        <table>
            <tr>
                <td><b>Name</b></td>
                <td>{schema.name}</td>
            </tr>
            <tr>
                <td><b>ID</b></td>
                <td>{schema.id}</td>
            </tr>
        </table>
    </div>
);


export default Schemas;
