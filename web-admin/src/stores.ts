import { get, writable } from "svelte/store";
import { notLoaded, RemoteData, Schema} from "./models";
import { getSdk, Sdk } from "./generated/graphql";
import { GraphQLClient } from "graphql-request";

const DEFAULT_GRAPHQL_ENDPOINT = "http://localhost:50106/graphql";

export const schemas = writable<RemoteData<Schema[]>>(notLoaded);
export const apiUrl = writable(
  localStorage.getItem("api-url") || initApiUrl()
);
export const graphqlClient = writable(newGraphqlClient());
export const darkMode = writable(localStorage.getItem("dark-mode") === "true");

apiUrl.subscribe((url) => {
  localStorage.setItem("api-url", url);
});

apiUrl.subscribe((url) => {
  graphqlClient.set(newGraphqlClient(url));
});

graphqlClient.subscribe((_) => {
  schemas.set(notLoaded)
})

darkMode.subscribe((isDarkMode) => {
  localStorage.setItem("dark-mode", JSON.stringify(isDarkMode));
});

function initApiUrl() {
  localStorage.setItem("api-url", DEFAULT_GRAPHQL_ENDPOINT);
  return DEFAULT_GRAPHQL_ENDPOINT;
}

export function newGraphqlClient(url: string = null): Sdk {
  const graphqlUrl = url == null ? get(apiUrl) : url;
  const client = new GraphQLClient(graphqlUrl);
  return getSdk(client);
}
