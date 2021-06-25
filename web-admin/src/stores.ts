import { writable } from "svelte/store";

const DEFAULT_GRAPHQL_ENDPOINT = "http://localhost:50106/graphql";

export const apiUrl = writable(localStorage.getItem("api-url") || initApiUrl());
export const darkMode = writable(localStorage.getItem("dark-mode") === "true");

apiUrl.subscribe((url) => {
  localStorage.setItem("api-url", url);
});

darkMode.subscribe((isDarkMode) => {
  localStorage.setItem("dark-mode", JSON.stringify(isDarkMode));
});

function initApiUrl() {
  localStorage.setItem("api-url", DEFAULT_GRAPHQL_ENDPOINT);
  return DEFAULT_GRAPHQL_ENDPOINT;
}
